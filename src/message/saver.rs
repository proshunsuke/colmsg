use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Barrier, Mutex,
};
use std::thread;

use chrono::NaiveDateTime;
use rayon::prelude::*;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

use crate::controller::{MemberStatus, ProgressEvent, ProgressSender, Service};
use crate::http::timeline::Timeline;
use crate::{
    errors::*,
    http::{self, client::SHNClient, groups::Groups, tags::Tags, timeline::TimelineMessages},
    message,
    message::file::{Picture, SaveToFile, Text, Video, Voice},
    Config, Kind,
};

lazy_static! {
    static ref ID_DATE_REGEX: Regex = Regex::new(r"(?x)(?P<id>\d+)_\d_(?P<date>\d+)").unwrap();
}

pub struct Saver<'a, C: SHNClient> {
    config: &'a Config<'a, C>,
}

impl<'b, C: SHNClient> Saver<'b, C> {
    pub fn new<'a>(config: &'a Config<C>) -> Saver<'a, C> {
        Saver { config }
    }

    pub fn save(&self) -> Result<()> {
        self.save_members_with_limit(4, None)
    }

    pub fn save_with_progress(
        &self,
        jobs: usize,
        service: Service,
        progress: &ProgressSender,
    ) -> Result<()> {
        self.save_members_with_limit(jobs, Some((service, progress)))
    }

    fn save_members_with_limit(
        &self,
        jobs: usize,
        progress: Option<(Service, &ProgressSender)>,
    ) -> Result<()> {
        let all_members =
            http::members::request_all(self.config.client.clone(), &self.config.access_token)?;
        let all_members_map: HashMap<u32, String> =
            all_members.into_iter().map(|m| (m.id, m.name)).collect();
        let groups = http::groups::request(self.config.client.clone(), &self.config.access_token)?;
        let tags = http::tags::request(self.config.client.clone(), &self.config.access_token)?;

        let member_identifiers = self.subscribed_list(&groups, &tags);
        let total_members = member_identifiers.len();

        if total_members == 0 {
            return Ok(());
        }

        let worker_count = jobs.min(total_members);
        let work_queue = Mutex::new(
            member_identifiers
                .into_iter()
                .enumerate()
                .collect::<VecDeque<_>>(),
        );
        let results = Mutex::new(Vec::with_capacity(total_members));
        let unauthorized_found = AtomicBool::new(false);
        let startup_barrier = Barrier::new(worker_count);
        thread::scope(|scope| {
            for _ in 0..worker_count {
                let work_queue = &work_queue;
                let results = &results;
                let startup_barrier = &startup_barrier;
                let all_members_map = &all_members_map;
                let unauthorized_found = &unauthorized_found;
                scope.spawn(move || {
                    let mut next =
                        next_member(work_queue, progress, total_members, unauthorized_found);
                    startup_barrier.wait();
                    while let Some((index, member_identifier)) = next {
                        let name = member_identifier.name.clone();
                        let result = self.save_messages(member_identifier, &all_members_map);
                        if matches!(
                            &result,
                            Err(Error::ReqwestError(request_error))
                                if request_error.status() == Some(reqwest::StatusCode::UNAUTHORIZED)
                        ) {
                            unauthorized_found.store(true, Ordering::Release);
                        }
                        if let Some((service, progress)) = progress {
                            let status = if result.is_ok() {
                                MemberStatus::Done
                            } else {
                                MemberStatus::Failed
                            };
                            let _ = progress.send(ProgressEvent::Member {
                                service,
                                index: index + 1,
                                total_members,
                                name: name.clone(),
                                status,
                            });
                        }
                        results.lock().unwrap().push((name, result));
                        next = next_member(work_queue, progress, total_members, unauthorized_found);
                    }
                });
            }
        });
        let results = results
            .into_inner()
            .map_err(|_| Error::Msg("member result queue was poisoned".to_owned()))?;

        let mut unauthorized_error = None;
        let mut member_errors = Vec::new();
        for (name, result) in results {
            if let Err(error) = result {
                if matches!(
                    &error,
                    Error::ReqwestError(request_error)
                        if request_error.status() == Some(reqwest::StatusCode::UNAUTHORIZED)
                ) {
                    if unauthorized_error.is_none() {
                        unauthorized_error = Some(error);
                    }
                } else {
                    member_errors.push(format!("{}: {}", name, error));
                }
            }
        }

        if let Some(error) = unauthorized_error {
            return Err(error);
        }
        if !member_errors.is_empty() {
            return Err(format!(
                "failed to save messages for members:\n{}",
                member_errors.join("\n")
            )
            .into());
        }

        Ok(())
    }

    fn subscribed_list(&self, group: &Vec<Groups>, tags: &Vec<Tags>) -> Vec<MemberIdentifier> {
        self.create_member_identifier_list(group, tags)
            .iter()
            .cloned()
            .filter(|m| m.subscription)
            .filter(|m| {
                if self.config.name.is_empty() {
                    return true;
                } // メンバー指定が無い場合は全メンバーを対象にする
                self.config.name.contains(&&*self.trim(&m.name))
            })
            .collect::<Vec<_>>()
    }

    fn create_member_identifier_list(
        &self,
        group: &Vec<Groups>,
        tags: &Vec<Tags>,
    ) -> Vec<MemberIdentifier> {
        let mut member_identifier_vec = Vec::with_capacity(group.len());
        group.iter().for_each(|g| {
            // もっといい書き方があるはず
            let mut group = "".to_string();
            let mut gen = "".to_string();
            tags.iter().for_each(|t| {
                let dimension = t.meta.as_ref().and_then(|meta| meta.dimension.as_ref());
                if g.tags.contains(&t.uuid) && dimension.is_some() {
                    group = t.name.clone();
                }
                if g.tags.contains(&t.uuid) && dimension.is_none() {
                    gen = t.name.clone();
                }
            });
            // 乃木坂の場合はg.tagsに世代情報(1期, 2期)が存在しないため全員乃木坂ディレクトリ以下に保存される
            member_identifier_vec.push(MemberIdentifier::new(
                g.id,
                self.trim(&g.name),
                gen,
                g.subscription.is_some(),
            ));
        });

        member_identifier_vec
    }

    fn trim(&self, str: &String) -> String {
        str.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    }

    fn save_messages(
        &self,
        member_identifier: MemberIdentifier,
        all_members_map: &HashMap<u32, String>,
    ) -> Result<()> {
        let member_dir_buf = self.create_member_dir_buf(&member_identifier)?;
        let mut id_dates = self.id_dates(&member_dir_buf);
        let mut fromdate = match self.config.from {
            Some(f) => f.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            None => self.latest_date(&id_dates)?,
        };

        // メンバー一覧を取得し、IDと名前のマップを作成
        let members = http::members::request(
            self.config.client.clone(),
            &self.config.access_token,
            &member_identifier.id,
        )?;
        let members_map: HashMap<u32, String> =
            members.into_iter().map(|m| (m.id, m.name)).collect();

        // 購読開始から24時間前までに配信されたメッセージを保存する
        let past_messages = http::past_messages::request(
            self.config.client.clone(),
            &self.config.access_token,
            &member_identifier.id,
        )?;
        for message in &past_messages.messages {
            self.save_message(
                &message,
                &id_dates,
                &member_dir_buf,
                &members_map,
                all_members_map,
            )?
        }
        id_dates = self.id_dates(&member_dir_buf);
        let mut count = http::timeline::DEFAULT_COUNT;

        // 購読しているメンバーのメッセージを取得するAPIを複数回叩くためのループ
        loop {
            let timeline = http::timeline::request(
                self.config.client.clone(),
                &self.config.access_token,
                &member_identifier.id,
                &fromdate,
                &count.to_string(),
            )?;

            let message_length = timeline.messages.len();

            // updated_atの値を基準にメッセージを取得している
            // 取得したメッセージのupdated_atがすべて同じだと基準が判明しない
            // 最新のメッセージまで取得出来たか、異なるupdated_atの値が現れるまでメッセージ取得数を増やしてメッセージ取得を施行する
            if message_length >= count && self.are_all_updated_at_same(&timeline) {
                count += http::timeline::DEFAULT_COUNT;
                continue;
            }

            // メッセージを取得するAPIを叩くと複数件のメッセージを取得出来る
            // そのメッセージを1件ずつ処理するためのループ
            for message in &timeline.messages {
                self.save_message(
                    &message,
                    &id_dates,
                    &member_dir_buf,
                    &members_map,
                    all_members_map,
                )?
            }

            // 最新のメッセージまで保存し終わったら終了する
            if message_length < count {
                break;
            };
            // 保存対象外のメッセージだけのページでも取得位置を進める。
            let next_date = timeline
                .messages
                .iter()
                .map(|message| &message.updated_at)
                .max()
                .unwrap();
            if next_date <= &fromdate {
                return Err("timeline did not advance".into());
            }
            fromdate = next_date.clone();
            id_dates = self.id_dates(&member_dir_buf);

            // 保存し終わったらメッセージ取得数をデフォルトに戻す
            count = http::timeline::DEFAULT_COUNT;
        }
        Ok(())
    }

    fn create_member_dir_buf(&self, member_identifier: &MemberIdentifier) -> Result<PathBuf> {
        let mut member_dir_buf = self.config.dir.clone();
        member_dir_buf.push(&member_identifier.gen);
        member_dir_buf.push(&member_identifier.name);
        if !member_dir_buf.is_dir() {
            fs::create_dir_all(&member_dir_buf)?
        }
        Ok(member_dir_buf)
    }

    fn save_message(
        &self,
        message: &TimelineMessages,
        id_dates: &Vec<IdDate>,
        member_dir_buf: &PathBuf,
        members_map: &HashMap<u32, String>,
        all_members_map: &HashMap<u32, String>,
    ) -> Result<()> {
        // 既に保存済のファイルはAPIリクエストしない&上書き保存せずスルー
        if id_dates
            .iter()
            .map(|id_date| id_date.id)
            .collect::<Vec<u32>>()
            .contains(&message.id)
        {
            return Ok(());
        }

        let poster_name = message
            .member_id
            .and_then(|id| {
                members_map
                    .get(&id)
                    .filter(|name| !name.trim().is_empty())
                    .or_else(|| {
                        all_members_map
                            .get(&id)
                            .filter(|name| !name.trim().is_empty())
                    })
            })
            .map(String::as_str)
            .unwrap_or("unknown");

        match message.messages_type.as_str() {
            "text" => {
                if !self.config.kind.contains(&Kind::Text) {
                    return Ok(());
                }
                let message_file_text = Text::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &0, &message.updated_at, poster_name)?,
                    &message.text,
                );
                message_file_text.save()?
            }
            "picture" => {
                if !self.config.kind.contains(&Kind::Picture) {
                    return Ok(());
                }
                let message_file_picture = Picture::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &1, &message.updated_at, poster_name)?,
                    &message.text,
                    &message.file,
                );
                message_file_picture.save()?
            }
            "video" => {
                if !self.config.kind.contains(&Kind::Video) {
                    return Ok(());
                }
                let message_file_video = Video::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &2, &message.updated_at, poster_name)?,
                    &message.file,
                );
                message_file_video.save()?
            }
            "voice" => {
                if !self.config.kind.contains(&Kind::Voice) {
                    return Ok(());
                }
                let message_file_voice = Voice::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &3, &message.updated_at, poster_name)?,
                    &message.file,
                );
                message_file_voice.save()?
            }
            "link" => {
                // リンク型はテキストファイルとして保存するが、種別は Link として扱う
                if !self.config.kind.contains(&Kind::Link) {
                    return Ok(());
                }
                let message_file_text = Text::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &4, &message.updated_at, poster_name)?,
                    &message.text,
                );
                message_file_text.save()?
            }
            _ => {
                let err = format!("unknown type: {}", message.messages_type.as_str());
                return Err(err.into());
            }
        };

        Ok(())
    }

    fn id_dates(&self, dir_buf: &PathBuf) -> Vec<IdDate> {
        let mut result = WalkDir::new(dir_buf)
            .into_iter()
            .par_bridge()
            .filter(|r| !r.as_ref().unwrap().path().is_dir())
            .map(|r| {
                let dir_entry = r.unwrap();
                dir_entry_to_id_date(&dir_entry)
            })
            .flatten()
            .collect::<Vec<_>>();
        result.sort_by(|a, b| a.id.cmp(&b.id));
        result
    }

    fn latest_date(&self, id_dates: &Vec<IdDate>) -> Result<String> {
        if id_dates.is_empty() {
            return Ok(String::from("2000-01-01T09:00:00Z"));
        }
        let date = id_dates.last().unwrap().clone().date;
        let date = NaiveDateTime::parse_from_str(&date, "%Y%m%d%H%M%S");
        Ok(date?.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }

    fn are_all_updated_at_same(&self, timeline: &Timeline) -> bool {
        let first_updated_at = &timeline.messages[0].updated_at;
        timeline
            .messages
            .iter()
            .all(|message| &message.updated_at == first_updated_at)
    }
}

fn next_member(
    work_queue: &Mutex<VecDeque<(usize, MemberIdentifier)>>,
    progress: Option<(Service, &ProgressSender)>,
    total_members: usize,
    unauthorized_found: &AtomicBool,
) -> Option<(usize, MemberIdentifier)> {
    let mut work_queue = work_queue.lock().unwrap();
    if unauthorized_found.load(Ordering::Acquire) {
        return None;
    }
    work_queue.pop_front().map(|(index, member_identifier)| {
        if let Some((service, progress)) = progress {
            let _ = progress.send(ProgressEvent::Member {
                service,
                index: index + 1,
                total_members,
                name: member_identifier.name.clone(),
                status: MemberStatus::Saving,
            });
        }
        (index, member_identifier)
    })
}

#[derive(Clone, Debug)]
pub struct MemberIdentifier {
    id: u32,
    name: String,
    gen: String,
    subscription: bool,
}

impl MemberIdentifier {
    pub fn new(id: u32, name: String, gen: String, subscription: bool) -> MemberIdentifier {
        MemberIdentifier {
            id,
            name,
            gen,
            subscription,
        }
    }
}

#[derive(Clone, Debug)]
struct IdDate {
    id: u32,
    date: String,
}

fn dir_entry_to_id_date(filename: &DirEntry) -> Option<IdDate> {
    let re = ID_DATE_REGEX.clone();
    re.captures(filename.file_name().to_str().unwrap())
        .and_then(|cap| {
            Some(IdDate {
                id: cap["id"].parse::<u32>().unwrap(),
                date: cap["date"].to_string(),
            })
        })
}

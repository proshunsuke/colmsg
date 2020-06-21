use std::fs;
use std::path::{PathBuf};

use regex::Regex;
use walkdir::{WalkDir, DirEntry};
use chrono::NaiveDateTime;

use crate::errors::*;
use crate::{Config, http, message, Group, Kind};
use crate::message::file::{Text, Picture, SaveToFile, Video, Voice};
use crate::http::groups::Groups;
use crate::http::tags::Tags;
use crate::http::timeline::TimelineMessages;

pub struct Saver<'a> {
    config: &'a Config<'a>
}

impl<'b> Saver<'b> {
    pub fn new<'a>(config: &'a Config) -> Saver<'a> {
        Saver { config }
    }

    pub fn save(&self) -> Result<()> {
        let groups = http::groups::request(&self.config.access_token)?;
        let tags = http::tags::request(&self.config.access_token)?;

        // TODO: 並列処理したい
        // 購読しているメンバー毎にメッセージを保存するためのループ
        for member_identifier in self.subscribed_list(&groups, &tags) {
            self.save_messages(member_identifier)?;
        }

        Ok(())
    }

    fn subscribed_list(&self, group: &Vec<Groups>, tags: &Vec<Tags>) -> Vec<MemberIdentifier> {
        self.create_member_identifier_list(group, tags)
            .iter()
            .cloned()
            .filter(|m| { m.subscription })
            .filter(|m| {
                match self.config.group {
                    Group::Keyakizaka => m.group == "欅坂46",
                    Group::Hinatazaka => m.group == "日向坂46",
                    Group::All => true
                }
            })
            .filter(|m| {
                if self.config.name.is_empty() { return true; } // メンバー指定が無い場合は全メンバーを対象にする
                self.config.name.contains(&&*self.trim(&m.name))
            })
            .collect::<Vec<_>>()
    }

    fn create_member_identifier_list(&self, group: &Vec<Groups>, tags: &Vec<Tags>) -> Vec<MemberIdentifier> {
        let mut member_identifier_vec = Vec::new();
        group.iter().for_each(|g| { // もっといい書き方があるはず
            let mut group = "".to_string();
            let mut gen = "".to_string();
            tags.iter().for_each(|t| {
                if g.tags.contains(&t.uuid) && t.meta.dimension.is_some() { group = t.name.clone(); }
                if g.tags.contains(&t.uuid) && t.meta.dimension.is_none() { gen = t.name.clone(); }
            });
            member_identifier_vec.push(MemberIdentifier::new(
                g.id, self.trim(&g.name), group, gen, g.subscription.is_some(),
            ));
        });

        member_identifier_vec
    }

    fn trim(&self, str: &String) -> String {
        str.chars().filter(|c| !c.is_whitespace()).collect::<String>()
    }

    fn save_messages(&self, member_identifier: MemberIdentifier) -> Result<()> {
        println!("saving messages of {}...", member_identifier.name);

        let member_dir_buf = self.create_member_dir_buf(&member_identifier)?;
        let id_dates = self.id_dates(&member_dir_buf);
        let mut fromdate = match self.config.from {
            Some(f) => f.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            None => self.latest_date(&id_dates)?
        };

        // 購読しているメンバーのメッセージを取得するAPIを複数回叩くためのループ
        loop {
            let timeline = http::timeline::request(&self.config.access_token, &member_identifier.id, &fromdate)?;

            // メッセージを取得するAPIを叩くと複数件のメッセージを取得出来る
            // そのメッセージを1件ずつ処理するためのループ
            for message in &timeline.messages {
                self.save_message(&message, &id_dates, &member_dir_buf)?
            };

            // 最新のメッセージまで保存し終わったら終了する
            if timeline.messages.len() < http::timeline::COUNT.parse().unwrap() { break; };
            let id_dates = self.id_dates(&member_dir_buf);
            fromdate = self.latest_date(&id_dates)?;
        }
        println!("complete saving messages of {}!", &member_identifier.name);

        Ok(())
    }

    fn create_member_dir_buf(&self, member_identifier: &MemberIdentifier) -> Result<PathBuf> {
        let mut member_dir_buf = self.config.dir.clone();
        member_dir_buf.push(&member_identifier.gen);
        member_dir_buf.push(&member_identifier.name);
        if !member_dir_buf.is_dir() {
            println!("create directory: {}", member_dir_buf.display());
            fs::create_dir_all(&member_dir_buf)?
        }
        Ok(member_dir_buf)
    }

    fn save_message(
        &self,
        message: &TimelineMessages,
        id_dates: &Vec<IdDate>,
        member_dir_buf: &PathBuf,
    ) -> Result<()> {
        // 既に保存済のファイルはAPIリクエストしない&上書き保存せずスルー
        if id_dates.iter().map(|id_date| id_date.id).collect::<Vec<u32>>().contains(&message.id) {
            return Ok(());
        }
        match message.messages_type.as_str() {
            "text" => {
                if !self.config.kind.contains(&Kind::Text) { return Ok(()); }
                let message_file_text = Text::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &0, &message.updated_at)?,
                    &message.text,
                );
                message_file_text.save()?
            },
            "picture" => {
                if !self.config.kind.contains(&Kind::Picture) { return Ok(()); }
                let message_file_picture = Picture::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &1, &message.updated_at)?,
                    &message.text,
                    &message.file,
                );
                message_file_picture.save()?
            },
            "video" => {
                if !self.config.kind.contains(&Kind::Video) { return Ok(()); }
                let message_file_video = Video::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &2, &message.updated_at)?,
                    &message.file,
                );
                message_file_video.save()?
            },
            "voice" => {
                if !self.config.kind.contains(&Kind::Voice) { return Ok(()); }
                let message_file_voice = Voice::new(
                    member_dir_buf,
                    message::file::file_name(&message.id, &3, &message.updated_at)?,
                    &message.file,
                );
                message_file_voice.save()?
            },
            _ => {
                let err = format!("unknown type: {}", message.messages_type.as_str());
                return Err(err.into());
            }
        };

        Ok(())
    }

    fn id_dates(&self, dir_buf: &PathBuf) -> Vec<IdDate> {
        WalkDir::new(dir_buf)
            .sort_by(move |a, b| {
                id_by_filename_format(&a).cmp(&id_by_filename_format(&b))
            })
            .into_iter()
            .filter(|r| !r.as_ref().unwrap().path().is_dir())
            .map(|r| {
                let dir_entry = r.unwrap();
                IdDate { id: id_by_filename_format(&dir_entry), date: date_by_filename_format(&dir_entry) }
            })
            .collect::<Vec<_>>()
    }

    fn latest_date(&self, id_dates: &Vec<IdDate>) -> Result<String> {
        if id_dates.is_empty() { return Ok(String::from("2000-01-01T09:00:00Z")); }
        let date = id_dates.last().unwrap().clone().date;
        let date = NaiveDateTime::parse_from_str(&date, "%Y%m%d%H%M%S");
        Ok(date?.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }
}

#[derive(Clone, Debug)]
pub struct MemberIdentifier {
    id: u32,
    name: String,
    group: String,
    gen: String,
    subscription: bool,
}

impl MemberIdentifier {
    pub fn new(id: u32, name: String, group: String, gen: String, subscription: bool) -> MemberIdentifier {
        MemberIdentifier { id, name, group, gen, subscription }
    }
}

#[derive(Clone, Debug)]
struct IdDate {
    id: u32,
    date: String,
}

fn id_by_filename_format(filename: &DirEntry) -> u32 {
    let re = Regex::new(r"(?x)(?P<id>\d+)_*").unwrap();
    let caps = &re.captures(filename.file_name().to_str().unwrap()).unwrap();
    caps["id"].parse::<u32>().unwrap()
}

fn date_by_filename_format(filename: &DirEntry) -> String {
    let re = Regex::new(r"(?x)\d+_\d_(?P<date>\d+)").unwrap();
    let caps = &re.captures(filename.file_name().to_str().unwrap()).unwrap();
    caps["date"].to_string()
}

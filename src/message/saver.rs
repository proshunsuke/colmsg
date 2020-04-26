use std::fs;
use std::path::{PathBuf, Path};

use regex::Regex;
use walkdir::WalkDir;
use chrono::NaiveDateTime;

use crate::errors::*;
use crate::{Config, http, message, Group, Kind};
use crate::message::file::{Text, Image, SaveToFile, Movie, Voice};
use crate::http::article_allhistory::ArticleAllhistoryResultHistory;
use crate::http::subscribe_list::SubscribeListResult;

const COUNT: u32 = 100;

pub struct Saver<'a> {
    config: &'a Config<'a>
}

impl<'b> Saver<'b> {
    pub fn new<'a>(config: &'a Config) -> Saver<'a> {
        Saver { config }
    }

    pub fn save(&self) -> Result<()> {
        let subscribe_list_res = http::subscribe_list::request(&self.config.token, &self.config.username)?;
        let result = subscribe_list_res.result;

        // TODO: 並列処理したい
        // 購読しているメンバー毎にメッセージを保存するためのループ
        for s in self.subscribed_list_by_result(&result) {
            let category_name = &s.categories.first().unwrap().category_name;
            let member_name = &s.groupname.chars().filter(|c| !c.is_whitespace()).collect::<String>();

            self.save_messages_by_member(category_name, member_name, &s.group)?;
        }
        Ok(())
    }

    fn subscribed_list_by_result<'a>(&self, result: &'a Vec<SubscribeListResult>) -> Vec<&'a SubscribeListResult> {
        result
            .iter()
            .filter(|r| {
                let category_name = &r.categories.first().unwrap().category_name;
                match self.config.group {
                    Group::Keyakizaka => category_name == "欅坂46",
                    Group::Hinatazaka => category_name == "日向坂46",
                    Group::All => true
                }
            })
            .filter(|r| {
                // メンバー指定が無い場合は全メンバーを対象にする
                if self.config.name.is_empty() { return true; }
                let member_name = &r.groupname.chars().filter(|c| !c.is_whitespace()).collect::<String>();
                let member_name = member_name.as_str();
                self.config.name.contains(&member_name)
            })
            .filter(|r| r.subscribed == "1")
            .collect::<Vec<_>>()
    }

    fn save_messages_by_member(
        &self,
        category_name: &String,
        member_name: &String,
        group: &String,
    ) -> Result<()> {
        println!("saving messages of {}...", member_name);

        let mut member_dir_buf = self.config.dir.clone();
        member_dir_buf.push(category_name);
        member_dir_buf.push(member_name);
        if !member_dir_buf.is_dir() {
            println!("create directory: {}", member_dir_buf.display());
            fs::create_dir_all(&member_dir_buf)?
        }

        let member_file_name_list = self.sorted_file_name_list_by_dir_buf(&member_dir_buf);
        let mut fromdate = match self.config.from {
            Some(f) => f.format("%Y/%m/%d %H:%M:%S").to_string(),
            None => self.latest_date_by_file_name_list(&member_file_name_list)?
        };

        let todate = match &self.config.to {
            Some(t) => t.format("%Y/%m/%d %H:%M:%S").to_string(),
            None => String::from("")
        };

        // 購読しているメンバーのメッセージを取得するAPIを複数回叩くためのループ
        loop {
            let article_allhistory_res = http::article_allhistory::request(
                group,
                &self.config.token,
                &self.config.username,
                COUNT,
                &fromdate,
                0,
                &todate,
            )?;

            let history = article_allhistory_res.result.history;

            // メッセージを取得するAPIを叩くと複数件のメッセージを取得出来る
            // そのメッセージを1件ずつ処理するためのループ
            for h in &history {
                self.save_message(&h, &member_file_name_list, &member_dir_buf)?;
            }

            // 最新のメッセージまで保存し終わったら終了する
            if history.len() < COUNT as usize { break; };
            let member_file_name_list = self.sorted_file_name_list_by_dir_buf(&member_dir_buf);
            fromdate = self.latest_date_by_file_name_list(&member_file_name_list)?;
        }
        println!("complete saving messages of {}!", member_name);
        Ok(())
    }

    fn save_message(
        &self,
        history: &ArticleAllhistoryResultHistory,
        member_file_name_list: &Vec<String>,
        member_dir_buf: &PathBuf,
    ) -> Result<()> {
        // この値が無い場合やkindが2の場合はletterなので保存せずスルー
        let seq_id = match &history.body.seq_id {
            Some(seq_id) => seq_id,
            None => return Ok(())
        };
        let media = match &history.body.media {
            Some(media) => media,
            None => return Ok(())
        };
        let talk = match &history.body.talk {
            Some(talk) => talk,
            None => return Ok(())
        };
        let contents = match &history.body.contents {
            Some(contents) => contents,
            None => return Ok(())
        };
        if history.kind == 2 { return Ok(()); }
        let file_name = message::file::file_name(seq_id, media, &history.body.date)?;
        // 既に保存済のファイルはAPIリクエストしない&上書き保存せずスルー
        if member_file_name_list.contains(&file_name) { return Ok(()); }
        match media {
            0 => {
                if !self.config.kind.contains(&Kind::Text) { return Ok(()); }
                let message_file_text = Text::new(
                    member_dir_buf,
                    file_name,
                    talk,
                );
                message_file_text.save()
            }
            1 => {
                if !self.config.kind.contains(&Kind::Image) { return Ok(()); }
                let message_file_image = Image::new(
                    member_dir_buf,
                    file_name,
                    talk,
                    contents,
                    &self.config.username,
                    &self.config.token,
                );
                message_file_image.save()
            }
            2 => {
                if !self.config.kind.contains(&Kind::Movie) { return Ok(()); }
                let message_file_movie = Movie::new(
                    member_dir_buf,
                    file_name,
                    contents,
                    &self.config.username,
                    &self.config.token,
                );
                message_file_movie.save()
            }
            3 => {
                if !self.config.kind.contains(&Kind::Voice) { return Ok(()); }
                let message_file_voice = Voice::new(
                    member_dir_buf,
                    file_name,
                    contents,
                    &self.config.username,
                    &self.config.token,
                );
                message_file_voice.save()
            }
            _ => {
                println!("unknown media type: {}", {media});
                Ok(())
            }
        }
    }

    fn sorted_file_name_list_by_dir_buf(&self, dir_buf: &PathBuf) -> Vec<String> {
        let re = Regex::new(r"(?x)(?P<seq_id>\d+)_*").unwrap();
        WalkDir::new(dir_buf).sort_by(move |a, b| {
            let capsa = &re.captures(&a.file_name().to_str().unwrap()).unwrap();
            let capsb = &re.captures(&b.file_name().to_str().unwrap()).unwrap();
            let seq_ida = &capsa["seq_id"].parse::<u32>().unwrap();
            let seq_idb = &capsb["seq_id"].parse::<u32>().unwrap();
            seq_ida.cmp(seq_idb)
        }).into_iter()
            .filter(|r| !r.as_ref().unwrap().path().is_dir())
            .map(|r| r.unwrap().file_name().to_str().unwrap().to_string())
            .map(|s| Path::new(&s).file_stem().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<_>>()
    }

    fn latest_date_by_file_name_list(&self, file_name_list: &Vec<String>) -> Result<String> {
        if file_name_list.is_empty() { return Ok(String::from("1970/01/01 09:00:00")) }
        let re = Regex::new(r"(?x)\d+_\d+_(?P<date>\d+)").unwrap();
        let caps = &re.captures(file_name_list.last().unwrap()).unwrap();
        let date = &caps["date"].parse::<String>().unwrap();
        let date = NaiveDateTime::parse_from_str(date, "%Y%m%d%H%M%S");
        Ok(date?.format("%Y/%m/%d %H:%M:%S").to_string())
    }
}

use std::io::{Write, copy};
use std::fs::File;
use std::io;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use reqwest::blocking::Response;

use crate::errors::*;
use crate::http;

pub struct Text<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    talk: &'a str,
}

impl Text<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        talk: &'a str
    ) -> Text<'a> {
        Text { member_dir_buf, file_name, talk }
    }
}

impl SaveToFile for Text<'_> {
    fn save(&self) -> Result<()> {
        save_text(self.member_dir_buf, &self.file_name, self.talk)?;
        Ok(())
    }
}

pub struct Image<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    talk: &'a str,
    contents: &'a str,
    username: &'a String,
    token: &'a String,
}

impl Image<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        talk: &'a str,
        contents: &'a str,
        username: &'a String,
        token: &'a String,
    ) -> Image<'a> {
        Image { member_dir_buf, file_name, talk, contents, username, token }
    }
}

impl SaveToFile for Image<'_> {
    fn save(&self) -> Result<()> {
        let article_res = http::article::request(&String::from(self.contents), &self.token, &self.username)?;

        if article_res.result.phoneimage.is_some() || article_res.result.silent.is_some() { return Ok(()); }

        let mut response = http::Client::new().get_request(&article_res.result.url)?;

        if !&self.talk.is_empty() { save_text(self.member_dir_buf, &self.file_name, self.talk)?};

        save_media(self.member_dir_buf, &self.file_name, &mut response, "jpg")?;
        Ok(())
    }
}

pub struct Movie<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    contents: &'a str,
    username: &'a String,
    token: &'a String,
}

impl Movie<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        contents: &'a str,
        username: &'a String,
        token: &'a String,
    ) -> Movie<'a> {
        Movie { member_dir_buf, file_name, contents, username, token }
    }
}

impl SaveToFile for Movie<'_> {
    fn save(&self) -> Result<()> {
        let article_res = http::article::request(&String::from(self.contents), &self.token, &self.username)?;

        if article_res.result.silent.is_none() { return Ok(()); }

        let mut response = http::Client::new().get_request(&article_res.result.url)?;
        save_media(self.member_dir_buf, &self.file_name, &mut response, "mp4")?;

        Ok(())
    }
}

pub struct Voice<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    contents: &'a str,
    username: &'a String,
    token: &'a String,
}

impl Voice<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        contents: &'a str,
        username: &'a String,
        token: &'a String,
    ) -> Voice<'a> {
        Voice {member_dir_buf, file_name, contents, username, token }
    }
}

impl SaveToFile for Voice<'_> {
    fn save(&self) -> Result<()> {
        let article_res = http::article::request(&String::from(self.contents), &self.token, &self.username)?;

        if article_res.result.phoneimage.is_none() { return Ok(()); }

        let mut response = http::Client::new().get_request(&article_res.result.url)?;
        save_media(self.member_dir_buf, &self.file_name, &mut response, "mp4")?;
        Ok(())
    }
}

pub trait SaveToFile {
    fn save(&self) -> Result<()>;
}

pub fn file_name<'a>(seq_id: &u32, media: &u32, date: &str) -> Result<String> {
    let date = NaiveDateTime::parse_from_str(date, "%Y/%m/%d %H:%M:%S");
    if let Err(e) = date {
        println!("parse error. date: {:#?}", date);
        return Err(e.into());
    }
    let date = date.unwrap()
        .format("%Y%m%d%H%M%S")
        .to_string();
    Ok(format!("{}_{}_{}", seq_id, media, &date))
}

fn save_text(member_dir_buf: &PathBuf, filename: &String, talk: &str) -> Result<()> {
    let mut file = create_file(member_dir_buf, &filename, "txt")?;
    let talk = talk.replace("\\r\\n", "\n");

    writeln!(file, "{}", talk)?;
    file.flush().unwrap();
    Ok(())
}

fn save_media(member_dir_buf: &PathBuf, filename: &String, response: &mut Response, extension: &str) -> Result<()> {
    let mut file = create_file(member_dir_buf, &filename, &extension)?;
    copy(response, &mut file)?;

    file.flush().unwrap();
    Ok(())
}

fn create_file(member_dir_buf: &PathBuf, filename: &str, extension: &str) -> io::Result<File> {
    let mut buf = member_dir_buf.clone();
    buf.push(filename);
    buf.set_extension(extension);
    File::create(&buf)
}

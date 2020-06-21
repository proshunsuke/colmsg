use std::io::{Write, copy};
use std::fs::File;
use std::io;
use std::path::PathBuf;

use chrono::NaiveDateTime;

use crate::errors::*;

pub struct Text<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    talk: &'a Option<String>,
}

impl Text<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        talk: &'a Option<String>,
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

pub struct Picture<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    talk: &'a Option<String>,
    file_url: &'a Option<String>,
}

impl Picture<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        talk: &'a Option<String>,
        file_url: &'a Option<String>,
    ) -> Picture<'a> {
        Picture { member_dir_buf, file_name, talk, file_url }
    }
}

impl SaveToFile for Picture<'_> {
    fn save(&self) -> Result<()> {
        save_media(self.member_dir_buf, &self.file_name, self.file_url, "jpg")?;
        save_text(self.member_dir_buf, &self.file_name, self.talk)?;
        Ok(())
    }
}

pub struct Video<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    file_url: &'a Option<String>,
}

impl Video<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        file_url: &'a Option<String>,
    ) -> Video<'a> {
        Video { member_dir_buf, file_name, file_url }
    }
}

impl SaveToFile for Video<'_> {
    fn save(&self) -> Result<()> {
        save_media(self.member_dir_buf, &self.file_name, self.file_url, "mp4")?;
        Ok(())
    }
}

pub struct Voice<'a> {
    member_dir_buf: &'a PathBuf,
    file_name: String,
    file_url: &'a Option<String>,
}

impl Voice<'_> {
    pub fn new<'a>(
        member_dir_buf: &'a PathBuf,
        file_name: String,
        file_url: &'a Option<String>,
    ) -> Voice<'a> {
        Voice { member_dir_buf, file_name, file_url }
    }
}

impl SaveToFile for Voice<'_> {
    fn save(&self) -> Result<()> {
        save_media(self.member_dir_buf, &self.file_name, self.file_url, "mp4")?;
        Ok(())
    }
}

pub trait SaveToFile {
    fn save(&self) -> Result<()>;
}

pub fn file_name<'a>(seq_id: &u32, media: &u32, date: &str) -> Result<String> {
    let parse_result = NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%SZ");
    if let Err(_e) = parse_result { return Err(format!("Parse error. date: {}", date).into()); }
    let date = parse_result
        .unwrap()
        .format("%Y%m%d%H%M%S")
        .to_string();
    Ok(format!("{}_{}_{}", seq_id, media, &date))
}

fn save_text(member_dir_buf: &PathBuf, filename: &String, talk: &Option<String>) -> Result<()> {
    if let Some(t) = talk {
        let mut file = create_file(member_dir_buf, &filename, "txt")?;
        let talk = t.replace("\\r\\n", "\n");

        writeln!(file, "{}", talk)?;
        file.flush()?;
    }
    Ok(())
}

fn save_media(member_dir_buf: &PathBuf, filename: &String, file_url: &Option<String>, extension: &str) -> Result<()> {
    if let Some(f) = file_url {
        let mut response = reqwest::blocking::get(f)?;
        let mut file = create_file(member_dir_buf, &filename, &extension)?;
        copy(&mut response, &mut file)?;

        file.flush()?;
    }
    Ok(())
}

fn create_file(member_dir_buf: &PathBuf, filename: &str, extension: &str) -> io::Result<File> {
    let mut buf = member_dir_buf.clone();
    buf.push(filename);
    buf.set_extension(extension);
    File::create(&buf)
}

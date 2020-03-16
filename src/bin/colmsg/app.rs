use std::path::PathBuf;
use std::fs;

use clap::ArgMatches;
use wild;

use colmsg::dirs::PROJECT_DIRS;

use crate::{
    clap_app,
    config::get_args_from_config_file,
};

pub struct App {
    pub matches: ArgMatches<'static>
}

use colmsg::{errors::*, Config, Group, Kind};
use chrono::NaiveDateTime;

impl App {
    pub fn new() -> Result<Self> {
        Ok(App {
            matches: Self::matches()?,
        })
    }

    fn matches() -> Result<ArgMatches<'static>> {
        let mut cli_args = wild::args_os();
        let mut args = get_args_from_config_file()
            .expect("Could not parse configuration file");

        args.insert(0, cli_args.next().unwrap());
        cli_args.for_each(|string| args.push(string));

        Ok(clap_app::build_app().get_matches_from(args))
    }

    pub fn config(&self) -> Result<Config> {
        let group = match self.matches.value_of("group") {
            Some("keyakizaka") => Group::Keyakizaka,
            Some("hinatazaka") => Group::Hinatazaka,
            _ => Group::All
        };

        let name = match self.matches.values_of("name") {
            Some(names) => {
                names
                    .map(|name| {name.trim()})
                    .collect::<Vec<_>>()
            },
            None => vec![]
        };

        let from = self.matches
            .value_of("from")
            .map(|from| NaiveDateTime::parse_from_str(from, "%Y/%m/%d %H:%M:%S"));
        let from = match from {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => return Err(e.into()),
            None => None
        };

        let to = self.matches
            .value_of("to")
            .map(|to| NaiveDateTime::parse_from_str(to, "%Y/%m/%d %H:%M:%S"));
        let to = match to {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => return Err(e.into()),
            None => None
        };

        let kind = match self.matches.values_of("kind") {
            Some(k) => {
                k.map(|v| {
                    match v {
                        "text" => Kind::Text,
                        "image" => Kind::Image,
                        "movie" => Kind::Movie,
                        "voice" | _ => Kind::Voice, // _ はあり得ないはずだが怒られるのでとりあえずVoiceにする
                    }
                }).collect::<Vec<_>>()
            }
            None => vec![Kind::Text, Kind::Image, Kind::Movie, Kind::Voice]
        };

        let dir = self.matches
            .value_of("dir")
            .map(PathBuf::from)
            .unwrap_or_else(|| PROJECT_DIRS.download_dir().to_path_buf());
        if !dir.is_dir() {
            println!("create directory: {}", dir.display());
            if let Err(e) = fs::create_dir_all(&dir) {
                return Err(e.into());
            }
        }

        let username = self.matches
            .value_of("username")
            .map(String::from)
            .unwrap_or_else(|| String::from("invalid_username"));

        let token = self.matches
            .value_of("token")
            .map(String::from)
            .unwrap_or_else(|| String::from("invalid_token"));

        Ok(Config { group, name, from, to, kind, dir, username, token })
    }
}

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate ansi_term;
extern crate chrono;
extern crate dirs as dirs_rs;
extern crate reqwest;
extern crate url;

pub mod controller;
pub mod dirs;
pub mod http;
mod message;

pub mod errors {
    error_chain! {
        foreign_links {
            ShellWordsParseError(::shell_words::ParseError);
            Clap(::clap::Error);
            Io(::std::io::Error);
            ParseError(::chrono::format::ParseError);
            UrlParseError(::url::ParseError);
            ReqwestError(::reqwest::Error);
            InvalidHeaderValue(::reqwest::header::InvalidHeaderValue);
        }
    }

    pub fn handle_error(error: &Error) {
        match error {
            Error(ErrorKind::Io(ref io_error), _)
            if io_error.kind() == ::std::io::ErrorKind::BrokenPipe =>
                {
                    ::std::process::exit(0);
                }
            _ => {
                use ansi_term::Colour::Red;
                eprintln!("{}: {}", Red.paint("[colmsg error]"), error);
            }
        };
    }
}

use std::path::PathBuf;

use chrono::{NaiveDateTime};
use crate::http::client::SHNClient;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Text,
    Picture,
    Video,
    Voice,
    Link,
}

pub struct Config<'a, C: SHNClient> {
    pub name: Vec<&'a str>,
    pub from: Option<NaiveDateTime>,
    pub kind: Vec<Kind>,
    pub dir: PathBuf,
    pub client: C,
    pub access_token: String,
}

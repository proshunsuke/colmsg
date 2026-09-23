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
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        ShellWordsParseError(#[from] ::shell_words::ParseError),
        #[error(transparent)]
        Clap(#[from] ::clap::Error),
        #[error(transparent)]
        Io(#[from] ::std::io::Error),
        #[error(transparent)]
        ParseError(#[from] ::chrono::format::ParseError),
        #[error(transparent)]
        UrlParseError(#[from] ::url::ParseError),
        #[error(transparent)]
        ReqwestError(#[from] ::reqwest::Error),
        #[error(transparent)]
        InvalidHeaderValue(#[from] ::reqwest::header::InvalidHeaderValue),
        #[error("{0}")]
        Msg(String),
    }

    impl From<String> for Error {
        fn from(message: String) -> Self {
            Self::Msg(message)
        }
    }

    impl From<&str> for Error {
        fn from(message: &str) -> Self {
            Self::Msg(message.to_owned())
        }
    }

    pub type Result<T> = ::std::result::Result<T, Error>;

    pub fn handle_error(error: &Error) {
        match error {
            Error::Io(io_error) if io_error.kind() == ::std::io::ErrorKind::BrokenPipe => {
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

use crate::http::client::SHNClient;
use chrono::NaiveDateTime;

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

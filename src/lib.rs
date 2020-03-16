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
mod http;
mod message;

pub mod errors {
    error_chain! {
        foreign_links {
            Clap(::clap::Error);
            Io(::std::io::Error);
            ParseError(::chrono::format::ParseError);
            ReqwestError(::reqwest::Error);
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Group {
    Keyakizaka,
    Hinatazaka,
    All
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Text,
    Image,
    Movie,
    Voice,
}

pub struct Config<'a> {
    pub group: Group,
    pub name: Vec<&'a str>,
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
    pub kind: Vec<Kind>,
    pub dir: PathBuf,
    pub username: String,
    pub token: String,
}

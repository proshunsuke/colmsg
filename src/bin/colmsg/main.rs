#[macro_use]
extern crate clap;

mod app;
mod clap_app;
pub mod config;

use std::process;
use std::io;
use std::io::Write;

use reqwest::StatusCode;

use crate::{
    app::App,
    config::delete_access_token_file,
};

use colmsg::dirs::PROJECT_DIRS;
use colmsg::{errors::*, Config};
use colmsg::errors::ErrorKind::ReqwestError;
use colmsg::controller::Controller;


fn run_controller(config: &Config) -> Result<bool> {
    let controller = Controller::new(config);
    controller.run()
}

fn run() -> Result<bool> {
    let app = App::new()?;
    let config = &app.config()?;

    if app.matches.is_present("config-dir") {
        writeln!(io::stdout(), "{}", PROJECT_DIRS.config_dir().to_string_lossy())?;
        Ok(true)
    } else if app.matches.is_present("download-dir") {
        writeln!(io::stdout(), "{}", PROJECT_DIRS.download_dir().to_string_lossy())?;
        Ok(true)
    } else {
        run_controller(&config)
    }
}

fn main() {
    let mut result = run();
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                if let Err(de) = delete_access_token_file() {
                    result = Err(de);
                    break;
                };
                result = run();
            }
            _ => { break; }
        }
    }

    match result {
        Err(error) => {
            handle_error(&error);
            process::exit(1);
        }
        Ok(false) => {
            process::exit(1);
        }
        Ok(true) => {
            process::exit(0);
        }
    }
}

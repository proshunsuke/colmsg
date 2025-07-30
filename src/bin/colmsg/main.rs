#[macro_use]
extern crate clap;

mod app;
mod clap_app;
pub mod config;

use std::{process, io, io::Write};

use reqwest::StatusCode;

use crate::{app::App, config::delete_access_token_file};

use colmsg::dirs::PROJECT_DIRS;
use colmsg::{errors::*, Config};
use colmsg::errors::ErrorKind::ReqwestError;
use colmsg::controller::Controller;
use colmsg::http::client::{SClient, SHNClient, HClient, NClient, AClient, MClient};

fn run_controller<C: SHNClient>(config: &Config<C>) -> Result<bool> {
    let controller = Controller::new(config);
    controller.run()
}

fn run_sakurazaka(app: &App) -> Result<bool> {
    if let None = app.matches.value_of("s_refresh_token") { return Ok(true) };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "sakurazaka"),
        None => true
    };
    if !is_run_by_group { return Ok(true) };
    let config: Config<SClient> = app.sakurazaka_config()?;
    run_controller(&config)
}

fn run_hinatazaka(app: &App) -> Result<bool> {
    if let None = app.matches.value_of("h_refresh_token") { return Ok(true) };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "hinatazaka"),
        None => true
    };
    if !is_run_by_group { return Ok(true) };
    let config: Config<HClient> = app.hinatazaka_config()?;
    run_controller(&config)
}

fn run_nogizaka(app: &App) -> Result<bool> {
    if let None = app.matches.value_of("n_refresh_token") { return Ok(true) };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "nogizaka"),
        None => true
    };
    if !is_run_by_group { return Ok(true) };
    let config: Config<NClient> = app.nogizaka_config()?;
    run_controller(&config)
}

fn run_asukasaito(app: &App) -> Result<bool> {
    if let None = app.matches.value_of("a_refresh_token") { return Ok(true) };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "asukasaito"),
        None => true
    };
    if !is_run_by_group { return Ok(true) };
    let config: Config<AClient> = app.asukasaito_config()?;
    run_controller(&config)
}

fn run_maishiraishi(app: &App) -> Result<bool> {
    if let None = app.matches.value_of("m_refresh_token") { return Ok(true) };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "maishiraishi"),
        None => true
    };
    if !is_run_by_group { return Ok(true) };
    let config: Config<MClient> = app.maishiraishi_config()?;
    run_controller(&config)
}

fn run() -> Result<bool> {
    let app = App::new()?;
    if app.matches.is_present("config-dir") {
        writeln!(io::stdout(), "{}", PROJECT_DIRS.config_dir().to_string_lossy())?;
        return Ok(true);
    }
    if app.matches.is_present("download-dir") {
        writeln!(io::stdout(), "{}", PROJECT_DIRS.download_dir().to_string_lossy())?;
        return Ok(true);
    }
    let mut result = run_sakurazaka(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_sakurazaka(&app);
            }
            _ => { break; }
        }
    }

    if let Err(_e) = &result { return result; }

    result = run_hinatazaka(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_hinatazaka(&app);
            }
            _ => { break; }
        }
    }

    if let Err(_e) = &result { return result; }

    let mut result = run_nogizaka(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_nogizaka(&app);
            }
            _ => { break; }
        }
    }

    if let Err(_e) = &result { return result; }

    let mut result = run_asukasaito(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_asukasaito(&app);
            }
            _ => { break; }
        }
    }

    if let Err(_e) = &result { return result; }

    let mut result = run_maishiraishi(&app);
    loop {
        match &result {
            Err(Error(ReqwestError(re), _)) => {
                if Some(StatusCode::UNAUTHORIZED) != re.status() { break; };
                delete_access_token_file()?;
                result = run_maishiraishi(&app);
            }
            _ => { break; }
        }
    }

    result
}

fn main() {
    let result = run();

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

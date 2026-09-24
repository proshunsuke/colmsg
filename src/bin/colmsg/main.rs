#[macro_use]
extern crate clap;

mod app;
mod clap_app;
pub mod config;

use std::{io, io::Write, process};

use reqwest::StatusCode;

use crate::{app::App, config::delete_access_token_file};

use colmsg::controller::Controller;
use colmsg::dirs::PROJECT_DIRS;
use colmsg::http::client::{AClient, HClient, MClient, NClient, SClient, SHNClient, YClient};
use colmsg::{errors::*, Config};

fn run_controller<C: SHNClient>(config: &Config<C>) -> Result<()> {
    let controller = Controller::new(config);
    controller.run()
}

fn run_sakurazaka(app: &App) -> Result<()> {
    let refresh_token = match app.matches.value_of("s_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "sakurazaka"),
        None => true,
    };
    if !is_run_by_group {
        return Ok(());
    };
    let config: Config<SClient> = app.sakurazaka_config(refresh_token)?;
    run_controller(&config)
}

fn run_hinatazaka(app: &App) -> Result<()> {
    let refresh_token = match app.matches.value_of("h_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "hinatazaka"),
        None => true,
    };
    if !is_run_by_group {
        return Ok(());
    };
    let config: Config<HClient> = app.hinatazaka_config(refresh_token)?;
    run_controller(&config)
}

fn run_nogizaka(app: &App) -> Result<()> {
    let refresh_token = match app.matches.value_of("n_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "nogizaka"),
        None => true,
    };
    if !is_run_by_group {
        return Ok(());
    };
    let config: Config<NClient> = app.nogizaka_config(refresh_token)?;
    run_controller(&config)
}

fn run_asukasaito(app: &App) -> Result<()> {
    let refresh_token = match app.matches.value_of("a_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "asukasaito"),
        None => true,
    };
    if !is_run_by_group {
        return Ok(());
    };
    let config: Config<AClient> = app.asukasaito_config(refresh_token)?;
    run_controller(&config)
}

fn run_maishiraishi(app: &App) -> Result<()> {
    let refresh_token = match app.matches.value_of("m_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "maishiraishi"),
        None => true,
    };
    if !is_run_by_group {
        return Ok(());
    };
    let config: Config<MClient> = app.maishiraishi_config(refresh_token)?;
    run_controller(&config)
}

fn run_yodel(app: &App) -> Result<()> {
    let refresh_token = match app.matches.value_of("y_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    let is_run_by_group = match app.matches.values_of("group") {
        Some(k) => k.clone().any(|v| v == "yodel"),
        None => true,
    };
    if !is_run_by_group {
        return Ok(());
    };
    let config: Config<YClient> = app.yodel_config(refresh_token)?;
    run_controller(&config)
}

fn run() -> Result<()> {
    let app = App::new()?;
    if app.matches.is_present("config-dir") {
        writeln!(
            io::stdout(),
            "{}",
            PROJECT_DIRS.config_dir().to_string_lossy()
        )?;
        return Ok(());
    }
    if app.matches.is_present("download-dir") {
        writeln!(
            io::stdout(),
            "{}",
            PROJECT_DIRS.download_dir().to_string_lossy()
        )?;
        return Ok(());
    }
    let mut result = run_sakurazaka(&app);
    if let Err(Error::ReqwestError(re)) = &result {
        if Some(StatusCode::UNAUTHORIZED) == re.status() {
            delete_access_token_file("s_access_token")?;
            result = run_sakurazaka(&app);
        }
    }

    if let Err(_e) = &result {
        return result;
    }

    result = run_hinatazaka(&app);
    if let Err(Error::ReqwestError(re)) = &result {
        if Some(StatusCode::UNAUTHORIZED) == re.status() {
            delete_access_token_file("h_access_token")?;
            result = run_hinatazaka(&app);
        }
    }

    if let Err(_e) = &result {
        return result;
    }

    let mut result = run_nogizaka(&app);
    if let Err(Error::ReqwestError(re)) = &result {
        if Some(StatusCode::UNAUTHORIZED) == re.status() {
            delete_access_token_file("n_access_token")?;
            result = run_nogizaka(&app);
        }
    }

    if let Err(_e) = &result {
        return result;
    }

    let mut result = run_asukasaito(&app);
    if let Err(Error::ReqwestError(re)) = &result {
        if Some(StatusCode::UNAUTHORIZED) == re.status() {
            delete_access_token_file("a_access_token")?;
            result = run_asukasaito(&app);
        }
    }

    if let Err(_e) = &result {
        return result;
    }

    let mut result = run_maishiraishi(&app);
    if let Err(Error::ReqwestError(re)) = &result {
        if Some(StatusCode::UNAUTHORIZED) == re.status() {
            delete_access_token_file("m_access_token")?;
            result = run_maishiraishi(&app);
        }
    }

    if let Err(_e) = &result {
        return result;
    }

    let mut result = run_yodel(&app);
    if let Err(Error::ReqwestError(re)) = &result {
        if Some(StatusCode::UNAUTHORIZED) == re.status() {
            delete_access_token_file("y_access_token")?;
            result = run_yodel(&app);
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
        Ok(()) => {
            process::exit(0);
        }
    }
}

#[macro_use]
extern crate clap;

mod app;
mod clap_app;
mod config;
mod progress;

use std::{
    io::{self, Write},
    process,
    sync::mpsc,
    thread,
    time::Duration,
};

use reqwest::StatusCode;

use crate::{app::App, config::delete_access_token_file, progress::ProgressDisplay};

use colmsg::controller::{Controller, ProgressEvent, ProgressSender, Service};
use colmsg::dirs::PROJECT_DIRS;
use colmsg::http::client::{AClient, HClient, MClient, NClient, SClient, SHNClient, YClient};
use colmsg::{errors::*, Config};

const PROGRESS_REFRESH_INTERVAL: Duration = Duration::from_millis(100);

fn run_controller<C: SHNClient>(
    config: &Config<C>,
    jobs: usize,
    service: Service,
    progress: &ProgressSender,
) -> Result<()> {
    Controller::new(config).run_with_progress(jobs, service, progress)
}

fn run_with_401_retry<F>(
    service: Service,
    token_file: &str,
    progress: &ProgressSender,
    mut run: F,
) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    let _ = progress.send(ProgressEvent::ServiceStarted { service });
    let result = run();
    if matches!(
        &result,
        Err(Error::ReqwestError(request_error))
            if request_error.status() == Some(StatusCode::UNAUTHORIZED)
    ) {
        let _ = progress.send(ProgressEvent::ServiceRetrying { service });
        delete_access_token_file(token_file)?;
        let _ = progress.send(ProgressEvent::ServiceStarted { service });
        return run();
    }
    result
}

fn run_sakurazaka(app: &App, jobs: usize, progress: &ProgressSender) -> Result<()> {
    let refresh_token = match app.matches.value_of("s_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    run_with_401_retry(Service::Sakurazaka, "s_access_token", progress, || {
        let config: Config<SClient> = app.sakurazaka_config(refresh_token)?;
        run_controller(&config, jobs, Service::Sakurazaka, progress)
    })
}

fn run_hinatazaka(app: &App, jobs: usize, progress: &ProgressSender) -> Result<()> {
    let refresh_token = match app.matches.value_of("h_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    run_with_401_retry(Service::Hinatazaka, "h_access_token", progress, || {
        let config: Config<HClient> = app.hinatazaka_config(refresh_token)?;
        run_controller(&config, jobs, Service::Hinatazaka, progress)
    })
}

fn run_nogizaka(app: &App, jobs: usize, progress: &ProgressSender) -> Result<()> {
    let refresh_token = match app.matches.value_of("n_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    run_with_401_retry(Service::Nogizaka, "n_access_token", progress, || {
        let config: Config<NClient> = app.nogizaka_config(refresh_token)?;
        run_controller(&config, jobs, Service::Nogizaka, progress)
    })
}

fn run_asukasaito(app: &App, jobs: usize, progress: &ProgressSender) -> Result<()> {
    let refresh_token = match app.matches.value_of("a_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    run_with_401_retry(Service::Asukasaito, "a_access_token", progress, || {
        let config: Config<AClient> = app.asukasaito_config(refresh_token)?;
        run_controller(&config, jobs, Service::Asukasaito, progress)
    })
}

fn run_maishiraishi(app: &App, jobs: usize, progress: &ProgressSender) -> Result<()> {
    let refresh_token = match app.matches.value_of("m_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    run_with_401_retry(Service::Maishiraishi, "m_access_token", progress, || {
        let config: Config<MClient> = app.maishiraishi_config(refresh_token)?;
        run_controller(&config, jobs, Service::Maishiraishi, progress)
    })
}

fn run_yodel(app: &App, jobs: usize, progress: &ProgressSender) -> Result<()> {
    let refresh_token = match app.matches.value_of("y_refresh_token") {
        Some(token) => token,
        None => return Ok(()),
    };
    run_with_401_retry(Service::Yodel, "y_access_token", progress, || {
        let config: Config<YClient> = app.yodel_config(refresh_token)?;
        run_controller(&config, jobs, Service::Yodel, progress)
    })
}

fn selected_services(app: &App) -> Vec<Service> {
    Service::ALL
        .iter()
        .copied()
        .filter(|service| {
            let token_argument = match service {
                Service::Sakurazaka => "s_refresh_token",
                Service::Hinatazaka => "h_refresh_token",
                Service::Nogizaka => "n_refresh_token",
                Service::Asukasaito => "a_refresh_token",
                Service::Maishiraishi => "m_refresh_token",
                Service::Yodel => "y_refresh_token",
            };
            if app.matches.value_of(token_argument).is_none() {
                return false;
            }
            match app.matches.values_of("group") {
                Some(groups) => groups.into_iter().any(|group| group == service.slug()),
                None => true,
            }
        })
        .collect()
}

fn run_service(app: &App, service: Service, jobs: usize, progress: &ProgressSender) -> Result<()> {
    match service {
        Service::Sakurazaka => run_sakurazaka(app, jobs, progress),
        Service::Hinatazaka => run_hinatazaka(app, jobs, progress),
        Service::Nogizaka => run_nogizaka(app, jobs, progress),
        Service::Asukasaito => run_asukasaito(app, jobs, progress),
        Service::Maishiraishi => run_maishiraishi(app, jobs, progress),
        Service::Yodel => run_yodel(app, jobs, progress),
    }
}

fn run_selected_services(
    app: &App,
    jobs: usize,
    display: &mut ProgressDisplay,
) -> Result<process::ExitCode> {
    let services = selected_services(app);
    if services.is_empty() {
        return Ok(process::ExitCode::SUCCESS);
    }

    let (progress, events) = mpsc::channel();
    let (results, display_error) = thread::scope(|scope| {
        let mut workers = Vec::with_capacity(services.len());
        for &service in &services {
            let progress = progress.clone();
            workers.push((
                service,
                scope.spawn(move || {
                    let result = run_service(app, service, jobs, &progress);
                    let _ = progress.send(ProgressEvent::ServiceFinished {
                        service,
                        success: result.is_ok(),
                    });
                    result
                }),
            ));
        }
        drop(progress);

        let mut display_error = None;
        loop {
            match events.recv_timeout(PROGRESS_REFRESH_INTERVAL) {
                Ok(event) => {
                    if let Err(error) = display.handle(event) {
                        if display_error.is_none() {
                            display_error = Some(error);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Err(error) = display.tick() {
                        if display_error.is_none() {
                            display_error = Some(error);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        let results = workers
            .into_iter()
            .map(|(service, worker)| {
                let result = match worker.join() {
                    Ok(result) => result,
                    Err(_) => Err(format!("{} worker panicked", service.name()).into()),
                };
                (service, result)
            })
            .collect::<Vec<_>>();
        (results, display_error)
    });

    let failed_services = results.iter().filter(|(_, result)| result.is_err()).count();
    let service_errors = results
        .into_iter()
        .filter_map(|(service, result)| {
            result
                .err()
                .map(|error| format!("{}: {}", service.name(), error))
        })
        .collect::<Vec<_>>();
    if !service_errors.is_empty() {
        handle_error(&Error::Msg(format!(
            "one or more services failed:\n{}",
            service_errors.join("\n")
        )));
    }
    display.print_summary(failed_services)?;
    if let Some(error) = display_error {
        return Err(error.into());
    }

    if failed_services == 0 {
        Ok(process::ExitCode::SUCCESS)
    } else {
        Ok(process::ExitCode::FAILURE)
    }
}

fn run() -> Result<process::ExitCode> {
    let app = App::new()?;
    if app.matches.is_present("config-dir") {
        writeln!(
            io::stdout(),
            "{}",
            PROJECT_DIRS.config_dir().to_string_lossy()
        )?;
        return Ok(process::ExitCode::SUCCESS);
    }
    if app.matches.is_present("download-dir") {
        writeln!(
            io::stdout(),
            "{}",
            PROJECT_DIRS.download_dir().to_string_lossy()
        )?;
        return Ok(process::ExitCode::SUCCESS);
    }

    let jobs = app
        .matches
        .value_of("jobs")
        .unwrap_or("4")
        .parse::<usize>()
        .map_err(|error| Error::Msg(format!("invalid --jobs value: {}", error)))?;
    let mut display = ProgressDisplay::new();
    run_selected_services(&app, jobs, &mut display)
}

fn main() -> process::ExitCode {
    let result = run();

    match result {
        Err(error) => {
            handle_error(&error);
            process::ExitCode::FAILURE
        }
        Ok(exit_code) => exit_code,
    }
}

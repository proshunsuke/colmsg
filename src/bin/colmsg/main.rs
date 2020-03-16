#[macro_use]
extern crate clap;

mod app;
mod clap_app;
pub mod config;

use std::process;

use crate::{app::App};

use colmsg::{errors::*, Config};
use colmsg::controller::Controller;

fn run_controller(config: &Config) -> Result<bool> {
    let controller = Controller::new(config);
    controller.run()
}

fn run() -> Result<bool> {
    let app = App::new()?;
    run_controller(&app.config()?)
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

use crate::errors::*;
use crate::Config;
use crate::message::saver::Saver;

pub struct Controller<'a> {
    config: &'a Config<'a>
}

impl<'b> Controller<'b> {
    pub fn new<'a>(config: &'a Config) -> Controller<'a> {
        Controller { config }
    }

    pub fn run(&self) -> Result<bool> {
        let no_errors: bool = true;

        let saver = Saver::new(self.config);
        saver.save()?;

        Ok(no_errors)
    }
}

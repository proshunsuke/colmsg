use crate::{errors::*, Config, message::saver::Saver, http::client::SHClient};

pub struct Controller<'a, C: SHClient> {
    config: &'a Config<'a, C>
}

impl<'b, C: SHClient> Controller<'b, C> {
    pub fn new<'a>(config: &'a Config<C>) -> Controller<'a, C> {
        Controller { config }
    }

    pub fn run(&self) -> Result<bool> {
        let no_errors: bool = true;

        let saver = Saver::new(self.config);
        saver.save()?;

        Ok(no_errors)
    }
}

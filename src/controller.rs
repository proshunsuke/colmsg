use crate::{errors::*, Config, message::saver::Saver, http::client::SHNClient};

pub struct Controller<'a, C: SHNClient> {
    config: &'a Config<'a, C>
}

impl<'b, C: SHNClient> Controller<'b, C> {
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

use crate::{errors::*, http::client::SHNClient, message::saver::Saver, Config};

pub struct Controller<'a, C: SHNClient> {
    config: &'a Config<'a, C>,
}

impl<'b, C: SHNClient> Controller<'b, C> {
    pub fn new<'a>(config: &'a Config<C>) -> Controller<'a, C> {
        Controller { config }
    }

    pub fn run(&self) -> Result<()> {
        let saver = Saver::new(self.config);
        saver.save()?;

        Ok(())
    }
}

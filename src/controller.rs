use std::sync::mpsc::Sender;

use crate::{errors::*, http::client::SHNClient, message::saver::Saver, Config};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Service {
    Sakurazaka,
    Hinatazaka,
    Nogizaka,
    Asukasaito,
    Maishiraishi,
    Yodel,
}

impl Service {
    pub const ALL: [Service; 6] = [
        Service::Sakurazaka,
        Service::Hinatazaka,
        Service::Nogizaka,
        Service::Asukasaito,
        Service::Maishiraishi,
        Service::Yodel,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Service::Sakurazaka => "Sakurazaka",
            Service::Hinatazaka => "Hinatazaka",
            Service::Nogizaka => "Nogizaka",
            Service::Asukasaito => "Asukasaito",
            Service::Maishiraishi => "Maishiraishi",
            Service::Yodel => "Yodel",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Service::Sakurazaka => "sakurazaka",
            Service::Hinatazaka => "hinatazaka",
            Service::Nogizaka => "nogizaka",
            Service::Asukasaito => "asukasaito",
            Service::Maishiraishi => "maishiraishi",
            Service::Yodel => "yodel",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemberStatus {
    Saving,
    Done,
    Failed,
}

#[derive(Clone, Debug)]
pub enum ProgressEvent {
    ServiceStarted {
        service: Service,
    },
    ServiceRetrying {
        service: Service,
    },
    ServiceFinished {
        service: Service,
        success: bool,
    },
    Member {
        service: Service,
        index: usize,
        total_members: usize,
        name: String,
        status: MemberStatus,
    },
}

pub type ProgressSender = Sender<ProgressEvent>;

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

    pub fn run_with_progress(
        &self,
        jobs: usize,
        service: Service,
        progress: &ProgressSender,
    ) -> Result<()> {
        if jobs == 0 {
            return Err("jobs must be greater than zero".into());
        }

        let saver = Saver::new(self.config);
        saver.save_with_progress(jobs, service, progress)
    }
}

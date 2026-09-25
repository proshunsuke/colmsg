use std::{collections::BTreeMap, io};

use colmsg::controller::{MemberStatus, ProgressEvent, Service};
use console::{measure_text_width, truncate_str, Term};

const PROGRESS_COUNT_MIN_WIDTH: usize = 2;
const SPINNER_FRAMES: [char; 4] = ['|', '/', '-', '\\'];

struct MemberProgress {
    total_members: usize,
    name: String,
    status: MemberStatus,
}

struct ServiceProgress {
    retrying: bool,
    failed: bool,
    members: BTreeMap<usize, MemberProgress>,
}

pub struct ProgressDisplay {
    term: Term,
    is_terminal: bool,
    rendered_lines: usize,
    spinner_frame: usize,
    services: BTreeMap<Service, ServiceProgress>,
}

impl ProgressDisplay {
    pub fn new() -> ProgressDisplay {
        let term = Term::stderr();
        let is_terminal = term.is_term();
        ProgressDisplay {
            term,
            is_terminal,
            rendered_lines: 0,
            spinner_frame: 0,
            services: BTreeMap::new(),
        }
    }

    pub fn handle(&mut self, event: ProgressEvent) -> io::Result<()> {
        match &event {
            ProgressEvent::ServiceStarted { service } => {
                self.services.insert(
                    *service,
                    ServiceProgress {
                        retrying: false,
                        failed: false,
                        members: BTreeMap::new(),
                    },
                );
            }
            ProgressEvent::ServiceRetrying { service } => {
                self.services
                    .entry(*service)
                    .or_insert_with(|| ServiceProgress {
                        retrying: true,
                        failed: false,
                        members: BTreeMap::new(),
                    })
                    .retrying = true;
            }
            ProgressEvent::ServiceFinished { service, success } => {
                let progress = self
                    .services
                    .entry(*service)
                    .or_insert_with(|| ServiceProgress {
                        retrying: false,
                        failed: false,
                        members: BTreeMap::new(),
                    });
                progress.retrying = false;
                progress.failed = !success;
            }
            ProgressEvent::Member {
                service,
                index,
                total_members,
                name,
                status,
            } => {
                let progress = self
                    .services
                    .entry(*service)
                    .or_insert_with(|| ServiceProgress {
                        retrying: false,
                        failed: false,
                        members: BTreeMap::new(),
                    });
                progress.members.insert(
                    *index,
                    MemberProgress {
                        total_members: *total_members,
                        name: name.clone(),
                        status: *status,
                    },
                );
            }
        }

        if self.is_terminal {
            self.render_terminal()
        } else {
            self.write_log_event(event)
        }
    }

    pub fn tick(&mut self) -> io::Result<()> {
        if !self.is_terminal || !self.has_active_members() {
            return Ok(());
        }

        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        self.render_terminal()
    }

    pub fn print_summary(&self, failed_services: usize) -> io::Result<()> {
        let (done_members, failed_members) = self.member_counts();
        let member_label = if done_members == 1 {
            "member"
        } else {
            "members"
        };
        let failed_member_label = if failed_members == 1 {
            "member"
        } else {
            "members"
        };
        let failed_service_label = if failed_services == 1 {
            "service"
        } else {
            "services"
        };

        let mut results = Vec::new();
        if done_members > 0 {
            results.push(format!("{done_members} {member_label} saved"));
        }
        if failed_members > 0 {
            results.push(format!("{failed_members} {failed_member_label} failed"));
        }
        if failed_services > 0 {
            results.push(format!("{failed_services} {failed_service_label} failed"));
        }

        let result = if results.is_empty() {
            "No matching members found.".to_owned()
        } else {
            format!("{}.", results.join("; "))
        };
        self.term.write_line(&result)
    }

    fn write_log_event(&self, event: ProgressEvent) -> io::Result<()> {
        match event {
            ProgressEvent::ServiceStarted { .. } => Ok(()),
            ProgressEvent::ServiceRetrying { service } => self.term.write_line(&format!(
                "{}: authorization expired; retrying",
                service.name()
            )),
            ProgressEvent::ServiceFinished { .. } => Ok(()),
            ProgressEvent::Member {
                service,
                index,
                total_members,
                name,
                status,
            } => self.term.write_line(&format!(
                "[{} {:>width$}/{:>width$}] {}: {}",
                service.name(),
                index,
                total_members,
                status_label(status),
                name,
                width = PROGRESS_COUNT_MIN_WIDTH
            )),
        }
    }

    fn render_terminal(&mut self) -> io::Result<()> {
        let mut lines = Vec::new();
        for service in Service::ALL.iter().copied() {
            let Some(progress) = self.services.get(&service) else {
                continue;
            };
            if progress.members.is_empty() {
                continue;
            }
            let suffix = if progress.failed {
                " (failed)"
            } else if progress.retrying {
                " (retrying)"
            } else {
                ""
            };
            lines.push(format!("{}{}", service.name(), suffix));
            for (index, member) in &progress.members {
                let spinner = if member.status == MemberStatus::Saving {
                    SPINNER_FRAMES[self.spinner_frame]
                } else {
                    ' '
                };
                lines.push(format!(
                    "  [{:>width$}/{:>width$}] {} {:10} {}",
                    index,
                    member.total_members,
                    spinner,
                    status_label(member.status),
                    member.name,
                    width = PROGRESS_COUNT_MIN_WIDTH
                ));
            }
        }

        if self.rendered_lines > 0 {
            self.term.clear_last_lines(self.rendered_lines)?;
        }

        let (_, terminal_width) = self.term.size();
        let width = usize::from(terminal_width).saturating_sub(1).max(1);
        for line in &lines {
            let truncated = truncate_str(line, width, "…");
            debug_assert!(measure_text_width(&truncated) <= width);
            self.term.write_line(&truncated)?;
        }
        self.rendered_lines = lines.len();
        Ok(())
    }

    fn has_active_members(&self) -> bool {
        self.services.values().any(|service| {
            service
                .members
                .values()
                .any(|member| member.status == MemberStatus::Saving)
        })
    }

    fn member_counts(&self) -> (usize, usize) {
        self.services
            .values()
            .flat_map(|service| service.members.values())
            .fold((0, 0), |(done, failed), member| match member.status {
                MemberStatus::Saving => (done, failed),
                MemberStatus::Done => (done + 1, failed),
                MemberStatus::Failed => (done, failed + 1),
            })
    }
}

fn status_label(status: MemberStatus) -> &'static str {
    match status {
        MemberStatus::Saving => "downloading",
        MemberStatus::Done => "done",
        MemberStatus::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_events_can_initialize_service_state_without_start_events() {
        let mut display = ProgressDisplay::new();
        display
            .handle(ProgressEvent::ServiceRetrying {
                service: Service::Sakurazaka,
            })
            .unwrap();
        display
            .handle(ProgressEvent::Member {
                service: Service::Hinatazaka,
                index: 1,
                total_members: 1,
                name: "処理中".to_owned(),
                status: MemberStatus::Saving,
            })
            .unwrap();
        display
            .handle(ProgressEvent::ServiceFinished {
                service: Service::Nogizaka,
                success: false,
            })
            .unwrap();

        display.print_summary(0).unwrap();
    }
}

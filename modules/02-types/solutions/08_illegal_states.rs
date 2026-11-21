//! # Make illegal states unrepresentable (reference solution)
//!
//! - The struct allowed 16 shapes; the enum allows exactly 3. The
//!   `unwrap_or("?")` fallbacks and `if self.assigned` checks are gone,
//!   because the cases they guarded against can't be written any more.
//! - Each variant holds the data that exists in that state: `agent` once
//!   assigned, `note` once resolved. Data every state has (`id`, `title`)
//!   stays on the struct.
//! - Moving data from one variant to the next needs ownership of it:
//!   `let Status::Assigned { agent } = &mut self.status else { .. }`, then
//!   `mem::take(agent)`. `match mem::replace(&mut self.status, Status::Open)`
//!   works too (module 01, exercise 05).
//! - Smells that point here: `bool` fields that can't both be true, `Option`
//!   fields that are `Some` exactly when a flag is set, comments like
//!   "only valid when ...".
//! - This checks transitions at run time (`assign` can return false). The
//!   typestate pattern, next, moves that check to compile time.

use std::mem;

#[derive(Debug, PartialEq)]
enum Status {
    Open,
    Assigned { agent: String },
    Resolved { agent: String, note: String },
}

#[derive(Debug)]
struct Ticket {
    id: u32,
    title: String,
    status: Status,
}

impl Ticket {
    fn new(id: u32, title: &str) -> Self {
        Ticket {
            id,
            title: title.to_string(),
            status: Status::Open,
        }
    }

    /// Assigns (or reassigns) an unresolved ticket to `agent`. Returns
    /// false, changing nothing, if the ticket is resolved.
    fn assign(&mut self, agent: &str) -> bool {
        match self.status {
            Status::Open | Status::Assigned { .. } => {
                self.status = Status::Assigned {
                    agent: agent.to_string(),
                };
                true
            }
            Status::Resolved { .. } => false,
        }
    }

    /// Resolves an assigned ticket with a note; the assigned agent is the
    /// one who resolved it. Returns false, changing nothing, if the ticket
    /// is open or already resolved.
    fn resolve(&mut self, note: &str) -> bool {
        let Status::Assigned { agent } = &mut self.status else {
            return false;
        };
        self.status = Status::Resolved {
            agent: mem::take(agent),
            note: note.to_string(),
        };
        true
    }

    /// Reopens a resolved ticket, assigned again to the agent who resolved
    /// it. The note is dropped. Returns false for unresolved tickets.
    fn reopen(&mut self) -> bool {
        let Status::Resolved { agent, .. } = &mut self.status else {
            return false;
        };
        self.status = Status::Assigned {
            agent: mem::take(agent),
        };
        true
    }

    /// The agent working on (or who resolved) the ticket.
    fn agent(&self) -> Option<&str> {
        match &self.status {
            Status::Open => None,
            Status::Assigned { agent } | Status::Resolved { agent, .. } => Some(agent),
        }
    }

    /// "#7 open: Printer on fire", "#7 with ana: Printer on fire" or
    /// "#7 resolved by ana: Printer on fire (unplugged it)".
    fn summary(&self) -> String {
        let Ticket { id, title, status } = self;
        match status {
            Status::Open => format!("#{id} open: {title}"),
            Status::Assigned { agent } => format!("#{id} with {agent}: {title}"),
            Status::Resolved { agent, note } => {
                format!("#{id} resolved by {agent}: {title} ({note})")
            }
        }
    }
}

fn main() {
    let mut ticket = Ticket::new(7, "Printer on fire");
    println!("{}", ticket.summary());
    ticket.assign("ana");
    println!("{}", ticket.summary());
    ticket.resolve("unplugged it");
    println!("{}", ticket.summary());
    ticket.reopen();
    println!("{} / {:?}", ticket.summary(), ticket.agent());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assigned(id: u32, agent: &str) -> Ticket {
        let mut ticket = Ticket::new(id, "Printer on fire");
        assert!(ticket.assign(agent));
        ticket
    }

    #[test]
    fn new_tickets_are_open() {
        let ticket = Ticket::new(7, "Printer on fire");
        assert_eq!(ticket.status, Status::Open);
        assert_eq!(ticket.agent(), None);
    }

    #[test]
    fn assigning_and_reassigning() {
        let mut ticket = assigned(7, "ana");
        assert_eq!(
            ticket.status,
            Status::Assigned {
                agent: "ana".to_string()
            }
        );
        assert!(ticket.assign("bo"));
        assert_eq!(ticket.agent(), Some("bo"));
    }

    #[test]
    fn resolving_needs_an_agent() {
        let mut ticket = Ticket::new(7, "Printer on fire");
        assert!(!ticket.resolve("fixed itself"));
        assert_eq!(ticket.status, Status::Open);
    }

    #[test]
    fn resolving() {
        let mut ticket = assigned(7, "ana");
        assert!(ticket.resolve("unplugged it"));
        assert_eq!(
            ticket.status,
            Status::Resolved {
                agent: "ana".to_string(),
                note: "unplugged it".to_string()
            }
        );
        assert_eq!(ticket.agent(), Some("ana"));
        assert!(!ticket.resolve("again"), "already resolved");
        assert!(!ticket.assign("bo"), "resolved tickets can't be reassigned");
        assert_eq!(ticket.agent(), Some("ana"));
    }

    #[test]
    fn reopening() {
        let mut ticket = assigned(7, "ana");
        assert!(!ticket.reopen(), "not resolved yet");
        ticket.resolve("unplugged it");
        assert!(ticket.reopen());
        assert_eq!(
            ticket.status,
            Status::Assigned {
                agent: "ana".to_string()
            }
        );
    }

    #[test]
    fn summaries() {
        let mut ticket = Ticket::new(7, "Printer on fire");
        assert_eq!(ticket.summary(), "#7 open: Printer on fire");
        ticket.assign("ana");
        assert_eq!(ticket.summary(), "#7 with ana: Printer on fire");
        ticket.resolve("unplugged it");
        assert_eq!(
            ticket.summary(),
            "#7 resolved by ana: Printer on fire (unplugged it)"
        );
    }

    #[test]
    fn states_are_built_directly() {
        let ticket = Ticket {
            id: 9,
            title: "VPN down".to_string(),
            status: Status::Resolved {
                agent: "cy".to_string(),
                note: "renewed cert".to_string(),
            },
        };
        assert_eq!(
            ticket.summary(),
            "#9 resolved by cy: VPN down (renewed cert)"
        );
    }
}

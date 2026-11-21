//! # Make illegal states unrepresentable
//!
//! A support ticket tracks its status with two flags and two `Option`s:
//! 16 combinations, of which 3 make sense (open, assigned, resolved with a
//! note), so every method defends against the other 13. Replace the four
//! fields with `status: Status`, an enum (deriving `Debug` and `PartialEq`)
//! whose variants carry exactly the data each state needs. The tests
//! already use it; read them for the names. No `.clone()`.

#[derive(Debug)]
struct Ticket {
    id: u32,
    title: String,
    assigned: bool,
    resolved: bool,
    agent: Option<String>,
    note: Option<String>,
}

impl Ticket {
    fn new(id: u32, title: &str) -> Self {
        Ticket {
            id,
            title: title.to_string(),
            assigned: false,
            resolved: false,
            agent: None,
            note: None,
        }
    }

    /// Assigns (or reassigns) an unresolved ticket to `agent`. Returns
    /// false, changing nothing, if the ticket is resolved.
    fn assign(&mut self, agent: &str) -> bool {
        if self.resolved {
            return false;
        }
        self.assigned = true;
        self.agent = Some(agent.to_string());
        true
    }

    /// Resolves an assigned ticket with a note; the assigned agent is the
    /// one who resolved it. Returns false, changing nothing, if the ticket
    /// is open or already resolved.
    fn resolve(&mut self, note: &str) -> bool {
        if !self.assigned || self.resolved {
            return false;
        }
        self.resolved = true;
        self.note = Some(note.to_string());
        true
    }

    /// Reopens a resolved ticket, assigned again to the agent who resolved
    /// it. The note is dropped. Returns false for unresolved tickets.
    fn reopen(&mut self) -> bool {
        if !self.resolved {
            return false;
        }
        self.resolved = false;
        self.note = None;
        true
    }

    /// The agent working on (or who resolved) the ticket.
    fn agent(&self) -> Option<&str> {
        if self.assigned {
            self.agent.as_deref()
        } else {
            None
        }
    }

    /// "#7 open: Printer on fire", "#7 with ana: Printer on fire" or
    /// "#7 resolved by ana: Printer on fire (unplugged it)".
    fn summary(&self) -> String {
        let agent = self.agent.as_deref().unwrap_or("?");
        if self.resolved {
            let note = self.note.as_deref().unwrap_or("");
            format!("#{} resolved by {agent}: {} ({note})", self.id, self.title)
        } else if self.assigned {
            format!("#{} with {agent}: {}", self.id, self.title)
        } else {
            format!("#{} open: {}", self.id, self.title)
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

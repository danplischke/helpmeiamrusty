//! # Moving out of `&mut`: take, replace, swap
//!
//! You can't move a value out from behind a `&mut` reference: that would
//! leave a hole in the struct. You *can* swap something else in, though.
//! `Option::take`, `std::mem::take` and `std::mem::replace` do exactly that.
//! No `.clone()`.

use std::mem;

#[derive(Debug, PartialEq)]
enum Connection {
    Idle,
    Connected { session: String, unsent: Vec<u8> },
    Closed { reason: String },
}

#[derive(Debug)]
struct Client {
    state: Connection,
    pending: Option<String>,
    history: Vec<String>,
}

impl Client {
    /// Hands back the pending request, leaving `None` behind.
    fn take_pending(&mut self) -> Option<String> {
        let request = self.pending;
        self.pending = None;
        request
    }

    /// Hands back the whole history, leaving an empty one behind.
    fn drain_history(&mut self) -> Vec<String> {
        let history = self.history;
        self.history = Vec::new();
        history
    }

    /// From `Connected`: switches to `Closed { reason: "<session> closed" }`
    /// and returns the unsent bytes. In any other state nothing changes and
    /// it returns `None`.
    fn disconnect(&mut self) -> Option<Vec<u8>> {
        match self.state {
            Connection::Connected { session, unsent } => {
                self.state = Connection::Closed {
                    reason: format!("{session} closed"),
                };
                Some(unsent)
            }
            _ => None,
        }
    }
}

fn connected_client() -> Client {
    Client {
        state: Connection::Connected {
            session: "s-42".to_string(),
            unsent: vec![1, 2, 3],
        },
        pending: Some("GET /".to_string()),
        history: vec!["hello".to_string(), "auth".to_string()],
    }
}

fn main() {
    let mut client = connected_client();
    println!("{:?}", client.take_pending());
    println!("{:?}", client.drain_history());
    println!("{:?}", client.disconnect());
    println!("{client:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_pending_once() {
        let mut client = connected_client();
        assert_eq!(client.take_pending(), Some("GET /".to_string()));
        assert_eq!(client.take_pending(), None);
    }

    #[test]
    fn drains_history() {
        let mut client = connected_client();
        assert_eq!(client.drain_history(), ["hello", "auth"]);
        assert!(client.history.is_empty());
    }

    #[test]
    fn disconnects_once() {
        let mut client = connected_client();
        assert_eq!(client.disconnect(), Some(vec![1, 2, 3]));
        assert_eq!(
            client.state,
            Connection::Closed {
                reason: "s-42 closed".to_string()
            }
        );
        assert_eq!(client.disconnect(), None);
    }

    #[test]
    fn idle_clients_stay_idle() {
        let mut client = connected_client();
        client.state = Connection::Idle;
        assert_eq!(client.disconnect(), None);
        assert_eq!(client.state, Connection::Idle);
    }
}

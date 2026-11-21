//! # Moving out of `&mut`: take, replace, swap (reference solution)
//!
//! - `Option::take()` moves the value out and leaves `None`.
//! - `mem::take` does the same for any `Default` type: the Vec is replaced
//!   by an empty one, which doesn't allocate.
//! - `mem::replace` works for any type: put a placeholder in, take the old
//!   value out by value, then match on it freely. If it wasn't the state you
//!   wanted, put it back. (`mem::swap` exchanges two `&mut` places.)
//! - Another way for `disconnect`: `let Connection::Connected { session,
//!   unsent } = &mut self.state else { return None };`, then
//!   `mem::take(unsent)` and build the reason from `session`.

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
        self.pending.take()
    }

    /// Hands back the whole history, leaving an empty one behind.
    fn drain_history(&mut self) -> Vec<String> {
        mem::take(&mut self.history)
    }

    /// From `Connected`: switches to `Closed { reason: "<session> closed" }`
    /// and returns the unsent bytes. In any other state nothing changes and
    /// it returns `None`.
    fn disconnect(&mut self) -> Option<Vec<u8>> {
        match mem::replace(&mut self.state, Connection::Idle) {
            Connection::Connected { session, unsent } => {
                self.state = Connection::Closed {
                    reason: format!("{session} closed"),
                };
                Some(unsent)
            }
            other => {
                self.state = other;
                None
            }
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

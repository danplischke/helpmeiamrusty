//! # Changing a collection while walking it
//!
//! You can't remove from (or push to) a Vec while iterating over it: the
//! iterator holds a borrow. And even if you could, removing by index while
//! counting up skips elements. Use the tools built for this instead:
//! `retain`, `iter_mut`, `drain`, `extract_if`, or collect first.

#[derive(Debug, PartialEq)]
struct Session {
    id: u32,
    expires_at: u64,
}

/// Removes every expired session (`expires_at <= now`) and returns the
/// removed ids in their original order.
fn evict_expired(sessions: &mut Vec<Session>, now: u64) -> Vec<u32> {
    let mut evicted = Vec::new();
    for (i, session) in sessions.iter().enumerate() {
        if session.expires_at <= now {
            evicted.push(session.id);
            sessions.remove(i);
        }
    }
    evicted
}

/// Extends every session by `secs`.
fn extend_all(sessions: &mut [Session], secs: u64) {
    for i in 0..sessions.len() {
        sessions[i].expires_at += secs;
    }
}

fn sessions(spec: &[(u32, u64)]) -> Vec<Session> {
    spec.iter()
        .map(|&(id, expires_at)| Session { id, expires_at })
        .collect()
}

fn main() {
    let mut live = sessions(&[(1, 10), (2, 5), (3, 5), (4, 20)]);
    println!("evicted {:?}", evict_expired(&mut live, 5));
    extend_all(&mut live, 60);
    println!("{live:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_expired_sessions() {
        let mut live = sessions(&[(1, 10), (2, 5), (3, 20), (4, 5)]);
        assert_eq!(evict_expired(&mut live, 5), [2, 4]);
        assert_eq!(live, sessions(&[(1, 10), (3, 20)]));
    }

    #[test]
    fn evicts_neighbours_too() {
        let mut live = sessions(&[(1, 5), (2, 5), (3, 20)]);
        assert_eq!(evict_expired(&mut live, 5), [1, 2]);
        assert_eq!(live, sessions(&[(3, 20)]));
    }

    #[test]
    fn extends_everything() {
        let mut live = sessions(&[(1, 10), (2, 20)]);
        extend_all(&mut live, 5);
        assert_eq!(live, sessions(&[(1, 15), (2, 25)]));
    }
}

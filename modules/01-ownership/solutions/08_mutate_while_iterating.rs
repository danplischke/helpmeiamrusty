//! # Changing a collection while walking it (reference solution)
//!
//! - `Vec::extract_if` (Rust 1.87) removes the matching elements and yields
//!   them, in one pass. Before 1.87, `retain` with a closure that also
//!   records what it drops does the same:
//!   `sessions.retain(|s| { let keep = s.expires_at > now; if !keep { evicted.push(s.id) } keep })`.
//! - The original loop had a second bug hiding behind the borrow error:
//!   `remove(i)` shifts everything left, so the next element was skipped.
//! - `for session in sessions` over a `&mut [T]` yields `&mut T`. No indices,
//!   no bounds checks (clippy: `needless_range_loop`).

#[derive(Debug, PartialEq)]
struct Session {
    id: u32,
    expires_at: u64,
}

/// Removes every expired session (`expires_at <= now`) and returns the
/// removed ids in their original order.
fn evict_expired(sessions: &mut Vec<Session>, now: u64) -> Vec<u32> {
    sessions
        .extract_if(.., |s| s.expires_at <= now)
        .map(|s| s.id)
        .collect()
}

/// Extends every session by `secs`.
fn extend_all(sessions: &mut [Session], secs: u64) {
    for session in sessions {
        session.expires_at += secs;
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

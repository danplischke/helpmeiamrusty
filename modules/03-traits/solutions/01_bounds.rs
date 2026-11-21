//! # Trait bounds, and where to put them (reference solution)
//!
//! - A bound states what the body uses: `>` needs `PartialOrd`, `{}` and
//!   `to_string` need `Display`, a `HashMap` key needs `Eq + Hash`.
//!   `PartialOrd` rather than `Ord`, because floats are only `PartialOrd`
//!   (NaN is unordered).
//! - Without `T: Clone`, `item.clone()` on a `&T` still compiles: it clones
//!   the *reference*. The error surfaced later, as a `HashMap<&T, _>`.
//! - `where` clauses read better once there are several bounds, and they can
//!   also constrain types that aren't parameters (`where Vec<T>: Debug`).
//! - Put a bound on the method that needs it (`fn render(&self) where T:
//!   Display`), or on a separate `impl<T: Display>` block. A bound on the
//!   whole `impl` would make `Leaderboard<Player>` lose `new` and `leader`.
//!   Bounds on the struct definition itself are rarely worth it: every impl
//!   then has to repeat them.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

/// The largest item, or `None` if there are none. On ties, the first one.
/// Works for anything comparable, floats included.
fn largest<T: PartialOrd>(items: &[T]) -> Option<&T> {
    let mut best = items.first()?;
    for item in items {
        if item > best {
            best = item;
        }
    }
    Some(best)
}

/// The items' display forms, joined with ", ".
fn join_all<T: Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// How often each distinct item occurs. The map owns copies of the items.
fn tally<T>(items: &[T]) -> HashMap<T, usize>
where
    T: Eq + Hash + Clone,
{
    let mut counts = HashMap::new();
    for item in items {
        *counts.entry(item.clone()).or_insert(0) += 1;
    }
    counts
}

/// Scores, in the order they were recorded.
struct Leaderboard<T> {
    entries: Vec<(T, u32)>,
}

impl<T> Leaderboard<T> {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn record(&mut self, player: T, score: u32) {
        self.entries.push((player, score));
    }

    /// The player with the highest score; the earliest recorded on ties.
    fn leader(&self) -> Option<&T> {
        self.entries
            .iter()
            .rev()
            .max_by_key(|(_, score)| *score)
            .map(|(player, _)| player)
    }

    /// One "player: score" line per entry, highest score first. Equal
    /// scores stay in the order they were recorded.
    fn render(&self) -> String
    where
        T: Display,
    {
        let mut sorted: Vec<&(T, u32)> = self.entries.iter().collect();
        sorted.sort_by_key(|(_, score)| Reverse(*score));
        sorted
            .iter()
            .map(|(player, score)| format!("{player}: {score}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn main() {
    println!("{:?}", largest(&[0.5, 2.25, 1.0]));
    println!("{}", join_all(&[1, 2, 3]));
    println!("{:?}", tally(&["a", "b", "a"]).get("a"));
    let mut board = Leaderboard::new();
    board.record("ann", 30);
    board.record("bob", 50);
    println!("{:?}\n{}", board.leader(), board.render());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A player that can be compared but not printed.
    #[derive(Debug, PartialEq)]
    struct Player(u32);

    #[test]
    fn largest_works_for_anything_comparable() {
        assert_eq!(largest(&[3, 7, 2]), Some(&7));
        assert_eq!(largest(&["pear", "apple", "plum"]), Some(&"plum"));
        assert_eq!(largest(&[2.5, 9.75, 1.0]), Some(&9.75), "floats too");
        assert_eq!(largest::<u8>(&[]), None);
    }

    #[test]
    fn largest_keeps_the_first_of_equals() {
        let items = [String::from("x"), String::from("x")];
        let found = largest(&items).unwrap();
        assert!(std::ptr::eq(found, &items[0]));
    }

    #[test]
    fn joins_anything_printable() {
        assert_eq!(join_all(&[1, 2, 3]), "1, 2, 3");
        assert_eq!(join_all(&['a']), "a");
        assert_eq!(join_all::<f64>(&[]), "");
    }

    #[test]
    fn tallies_into_owned_keys() {
        let words = [String::from("ok"), String::from("no"), String::from("ok")];
        let counts: HashMap<String, usize> = tally(&words);
        assert_eq!(counts["ok"], 2);
        assert_eq!(counts.len(), 2);
        assert_eq!(tally(&[1, 1, 1]), HashMap::from([(1, 3)]));
    }

    #[test]
    fn leaderboard_of_unprintable_players() {
        let mut board = Leaderboard::new();
        board.record(Player(1), 30);
        board.record(Player(2), 50);
        board.record(Player(3), 50);
        assert_eq!(board.leader(), Some(&Player(2)));
        assert_eq!(Leaderboard::<Player>::new().leader(), None);
    }

    #[test]
    fn renders_printable_players() {
        let mut board = Leaderboard::new();
        board.record("ann", 30);
        board.record("bob", 50);
        board.record("cy", 30);
        assert_eq!(board.render(), "bob: 50\nann: 30\ncy: 30");
    }
}

//! # Equality, hashing, ordering and `Borrow` (reference solution)
//!
//! - The hidden bug: `#[derive(Hash)]` hashed the name as typed, so
//!   "Ferris" and "FERRIS" were equal but landed in different buckets.
//!   Hash exactly what `eq` compares (clippy: `derived_hash_with_manual_eq`).
//! - `Borrow<Q>` promises that the borrowed form compares and hashes like
//!   the owned one. True for `Sku` (its `String`), so `get("BOLT-M6")`
//!   works. False for `Username`: never give it `Borrow<str>`.
//! - `Index` returns a reference, so `&self.counts[sku]` borrows from the
//!   map, and `HashMap`'s own `Index` panics for a missing key.
//! - Ordering by a key: compare the most important part first, then
//!   `.then(..)` (or `.then_with(|| ..)`, which is lazy). Swapping `self`
//!   and `other` reverses one key. `(self.priority, Reverse(self.id))` as a
//!   tuple key works too.
//! - `PartialOrd` delegates to `Ord` with `Some(self.cmp(other))`. Two
//!   separate implementations can drift apart; clippy flags them
//!   (`non_canonical_partial_ord_impl`).

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A username. Comparisons ignore ASCII case: "Ferris" and "FERRIS" are the
/// same user. It still displays as typed.
#[derive(Debug, Clone)]
struct Username(String);

impl PartialEq for Username {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for Username {}

impl Hash for Username {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

/// A stock-keeping unit, like "BOLT-M6". Case matters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Sku(String);

/// Lets a `HashMap<Sku, _>` be searched with a plain `&str`, without
/// building a `Sku` first.
impl Borrow<str> for Sku {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Quantities on hand.
#[derive(Debug, Default)]
struct Stock {
    counts: HashMap<Sku, u32>,
}

impl Stock {
    /// Adds `qty` of `sku`, which may be new.
    fn add(&mut self, sku: &str, qty: u32) {
        match self.counts.get_mut(sku) {
            Some(count) => *count += qty,
            None => {
                self.counts.insert(Sku(sku.to_string()), qty);
            }
        }
    }
}

/// `stock["BOLT-M6"]` is the quantity of a SKU that has to be in stock.
/// Like indexing a `HashMap`, it panics if the SKU is unknown.
impl Index<&str> for Stock {
    type Output = u32;

    fn index(&self, sku: &str) -> &u32 {
        &self.counts[sku]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Low,
    Normal,
    Urgent,
}

/// A support ticket. A *greater* ticket gets worked on first: higher
/// priority wins, and among equal priorities the older ticket (the lower
/// id) wins. A `BinaryHeap<Ticket>` then pops tickets in working order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ticket {
    id: u32,
    priority: Priority,
}

impl Ord for Ticket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then(other.id.cmp(&self.id))
    }
}

impl PartialOrd for Ticket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let ferris = Username("Ferris".to_string());
    println!("{}", ferris == Username("FERRIS".to_string()));
    let mut stock = Stock::default();
    stock.add("BOLT-M6", 40);
    println!("{:?} {}", stock.counts.get("BOLT-M6"), stock["BOLT-M6"]);
    let mut tickets = vec![
        Ticket {
            id: 1,
            priority: Priority::Low,
        },
        Ticket {
            id: 2,
            priority: Priority::Urgent,
        },
    ];
    tickets.sort();
    println!("{tickets:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BinaryHeap, HashSet};

    fn user(name: &str) -> Username {
        Username(name.to_string())
    }

    fn ticket(id: u32, priority: Priority) -> Ticket {
        Ticket { id, priority }
    }

    #[test]
    fn usernames_ignore_ascii_case() {
        assert_eq!(user("Ferris"), user("FERRIS"));
        assert_ne!(user("Ferris"), user("Ferris2"));
        assert_eq!(user("Ferris").0, "Ferris", "kept as typed");
    }

    #[test]
    fn equal_usernames_hash_the_same() {
        let names = ["Ferris", "ferris", "FERRIS", "fErRiS", "Corro", "corro"];
        let set: HashSet<Username> = names.into_iter().map(user).collect();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&user("CORRO")));
    }

    #[test]
    fn stock_is_searched_with_str() {
        let mut stock = Stock::default();
        stock.add("BOLT-M6", 40);
        stock.add("NUT-M6", 25);
        stock.add("BOLT-M6", 10);
        assert_eq!(stock.counts.get("BOLT-M6"), Some(&50));
        assert_eq!(stock.counts.get("bolt-m6"), None, "SKUs are case-sensitive");
        assert_eq!(stock["NUT-M6"], 25);
        assert_eq!(stock.counts.len(), 2);
    }

    #[test]
    #[should_panic]
    fn indexing_an_unknown_sku_panics() {
        let stock = Stock::default();
        let _ = stock["GEAR-12"];
    }

    #[test]
    fn more_urgent_tickets_are_greater() {
        assert!(ticket(9, Priority::Urgent) > ticket(1, Priority::Normal));
        assert!(ticket(1, Priority::Normal) > ticket(2, Priority::Normal));
        let low = ticket(3, Priority::Low);
        assert_eq!(low.cmp(&low), Ordering::Equal);
    }

    #[test]
    fn a_heap_pops_tickets_in_working_order() {
        let mut queue = BinaryHeap::from([
            ticket(1, Priority::Low),
            ticket(2, Priority::Urgent),
            ticket(3, Priority::Normal),
            ticket(4, Priority::Urgent),
            ticket(5, Priority::Normal),
        ]);
        let mut order = Vec::new();
        while let Some(next) = queue.pop() {
            order.push(next.id);
        }
        assert_eq!(order, [2, 4, 3, 5, 1]);
    }
}

//! # Equality, hashing, ordering and `Borrow`
//!
//! Derived `PartialEq`, `Hash` and `Ord` look at every field, in order.
//! When you need something else, write the impls yourself and keep them
//! consistent: if `a == b`, then `a` and `b` must hash the same, and `Ord`
//! must agree with `==`. `HashMap<K, V>::get` accepts any `&Q` where
//! `K: Borrow<Q>`, which is how a `HashMap<String, V>` can be searched with
//! a `&str`.
//!
//! Fill in the trait methods. One test will still fail afterwards, for a
//! reason outside the function bodies.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Index;

/// A username. Comparisons ignore ASCII case: "Ferris" and "FERRIS" are the
/// same user. It still displays as typed.
#[derive(Debug, Clone, Hash)]
struct Username(String);

impl PartialEq for Username {
    fn eq(&self, other: &Self) -> bool {
        todo!("is {self:?} the same user as {other:?}?")
    }
}

impl Eq for Username {}

/// A stock-keeping unit, like "BOLT-M6". Case matters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Sku(String);

/// Lets a `HashMap<Sku, _>` be searched with a plain `&str`, without
/// building a `Sku` first.
impl Borrow<str> for Sku {
    fn borrow(&self) -> &str {
        todo!("{self:?} as a &str")
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
        todo!("add {qty} of {sku}")
    }
}

/// `stock["BOLT-M6"]` is the quantity of a SKU that has to be in stock.
/// Like indexing a `HashMap`, it panics if the SKU is unknown.
impl Index<&str> for Stock {
    type Output = u32;

    fn index(&self, sku: &str) -> &u32 {
        todo!("the quantity of {sku}")
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
        todo!("compare {self:?} with {other:?}")
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

//! # The entry API (and a borrow checker blind spot) (reference solution)
//!
//! - The rejected `get_or_compute` is "NLL problem case #3": conditionally
//!   returning a borrow from a function. Polonius, the next-generation
//!   borrow checker, accepts it; until it ships, restructure.
//! - `entry(key)` does the lookup once and hands back a slot:
//!   `or_insert(v)`, `or_insert_with(|| ..)` (computed only if missing),
//!   `or_default()`, `and_modify(|v| ..)`. It returns `&mut V` into the map.
//! - `contains_key` + `get_mut(..).unwrap()` + `insert` is two lookups and
//!   an `unwrap` that can't fail but still has to be read and trusted.

use std::collections::{BTreeMap, HashMap};

/// Returns the cached value for `key`, computing and caching it if missing.
fn get_or_compute(
    cache: &mut HashMap<u64, String>,
    key: u64,
    compute: fn(u64) -> String,
) -> &String {
    cache.entry(key).or_insert_with(|| compute(key))
}

/// Counts how often each word occurs.
fn tally<'a>(words: &[&'a str]) -> HashMap<&'a str, usize> {
    let mut counts = HashMap::new();
    for &word in words {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

/// Groups words by length, keeping their original order within a group.
fn group_by_len<'a>(words: &[&'a str]) -> BTreeMap<usize, Vec<&'a str>> {
    let mut groups: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for &word in words {
        groups.entry(word.len()).or_default().push(word);
    }
    groups
}

fn square_text(n: u64) -> String {
    (n * n).to_string()
}

fn main() {
    let mut cache = HashMap::new();
    println!("{}", get_or_compute(&mut cache, 12, square_text));
    let words = ["to", "be", "or", "not", "to", "be"];
    println!("{:?}", tally(&words));
    println!("{:?}", group_by_len(&words));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_then_caches() {
        let mut cache = HashMap::from([(3, "cached".to_string())]);
        assert_eq!(get_or_compute(&mut cache, 3, square_text), "cached");
        assert_eq!(get_or_compute(&mut cache, 4, square_text), "16");
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn tallies() {
        let counts = tally(&["a", "b", "a", "c", "a"]);
        assert_eq!(counts, HashMap::from([("a", 3), ("b", 1), ("c", 1)]));
    }

    #[test]
    fn groups_in_order() {
        let groups = group_by_len(&["hi", "hey", "yo", "hello", "sup"]);
        assert_eq!(
            groups,
            BTreeMap::from([
                (2, vec!["hi", "yo"]),
                (3, vec!["hey", "sup"]),
                (5, vec!["hello"]),
            ])
        );
    }
}

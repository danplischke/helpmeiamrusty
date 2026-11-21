//! # The entry API (and a borrow checker blind spot)
//!
//! `get_or_compute` is correct code that today's borrow checker still
//! rejects: it can't see that the early `return` ends the first borrow on
//! every other path. The fix, and the idiomatic way to write "look up, or
//! insert", is the entry API.
//!
//! `tally` and `group_by_len` already work, but each does two lookups where
//! one would do. Rewrite them with `entry` as well. The checker can't tell
//! one lookup from two, so that part is on your honour.

use std::collections::{BTreeMap, HashMap};

/// Returns the cached value for `key`, computing and caching it if missing.
fn get_or_compute(
    cache: &mut HashMap<u64, String>,
    key: u64,
    compute: fn(u64) -> String,
) -> &String {
    if let Some(value) = cache.get(&key) {
        return value;
    }
    cache.insert(key, compute(key));
    cache.get(&key).unwrap()
}

/// Counts how often each word occurs.
fn tally<'a>(words: &[&'a str]) -> HashMap<&'a str, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for &word in words {
        if counts.contains_key(word) {
            *counts.get_mut(word).unwrap() += 1;
        } else {
            counts.insert(word, 1);
        }
    }
    counts
}

/// Groups words by length, keeping their original order within a group.
fn group_by_len<'a>(words: &[&'a str]) -> BTreeMap<usize, Vec<&'a str>> {
    let mut groups: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for &word in words {
        match groups.get_mut(&word.len()) {
            Some(group) => group.push(word),
            None => {
                groups.insert(word.len(), vec![word]);
            }
        }
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

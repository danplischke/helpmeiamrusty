//! # Beyond Vec and HashMap (reference solution)
//!
//! - A sliding window is a `VecDeque`: `push_back` the new item,
//!   `pop_front` the oldest. Keeping a running total makes the average
//!   O(1). `Vec::remove(0)` would shift every element instead.
//! - Top-k with a *min*-heap of size k: push each item, and when the heap
//!   holds k + 1, pop the smallest. That's O(n log k) time and O(k) space.
//!   `into_sorted_vec` sorts ascending by `Reverse`, so slowest first.
//! - `BTreeMap::range(a..b)` finds the start in O(log n), then walks in key
//!   order. It **panics** if the start is after the end, hence the guard.
//! - Set operations return lazy iterators of references; collecting them
//!   into a `BTreeSet` gives a sorted, deterministic result. Never assert
//!   on the iteration order of a `HashSet`.
//! - `sort_by_key` calls the key function twice per comparison, so
//!   O(n log n) times. `sort_by_cached_key` calls it once per element and
//!   sorts the cached keys: use it when the key allocates or is slow.

use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashSet, VecDeque};

/// The last `cap` response times, and their running total.
#[derive(Debug)]
struct Recent {
    cap: usize,
    times: VecDeque<u32>,
    total: u64,
}

impl Recent {
    fn new(cap: usize) -> Recent {
        Recent {
            cap,
            times: VecDeque::with_capacity(cap + 1),
            total: 0,
        }
    }

    /// Records a response time. If that makes more than `cap` stored
    /// times, drops the oldest one and returns it.
    fn record(&mut self, ms: u32) -> Option<u32> {
        self.times.push_back(ms);
        self.total += u64::from(ms);
        if self.times.len() <= self.cap {
            return None;
        }
        let oldest = self.times.pop_front()?;
        self.total -= u64::from(oldest);
        Some(oldest)
    }

    /// The average of the stored times, without walking them; `None` when
    /// nothing is stored.
    fn average(&self) -> Option<f64> {
        if self.times.is_empty() {
            return None;
        }
        Some(self.total as f64 / self.times.len() as f64)
    }
}

/// The `k` slowest times, slowest first. Keeps a heap of at most `k`
/// times rather than sorting all of them.
fn slowest(times: &[u32], k: usize) -> Vec<u32> {
    let mut heap = BinaryHeap::with_capacity(k + 1);
    for &time in times {
        heap.push(Reverse(time));
        if heap.len() > k {
            heap.pop();
        }
    }
    heap.into_sorted_vec()
        .into_iter()
        .map(|Reverse(time)| time)
        .collect()
}

/// The versions deployed at times `from <= t < to`, in time order. Empty
/// when `from >= to`.
fn deploys_between(deploys: &BTreeMap<u64, String>, from: u64, to: u64) -> Vec<&str> {
    if from >= to {
        return Vec::new();
    }
    deploys
        .range(from..to)
        .map(|(_, version)| version.as_str())
        .collect()
}

/// The hosts that appeared since the last scan, and the hosts that
/// disappeared.
fn churn<'a>(
    before: &HashSet<&'a str>,
    after: &HashSet<&'a str>,
) -> (BTreeSet<&'a str>, BTreeSet<&'a str>) {
    let appeared = after.difference(before).copied().collect();
    let gone = before.difference(after).copied().collect();
    (appeared, gone)
}

thread_local! {
    /// How many times `domain_key` has run on this thread (for the tests).
    static KEY_CALLS: Cell<usize> = const { Cell::new(0) };
}

/// The sort key for a host: its labels reversed, so "api.example.com"
/// becomes "com.example.api". It allocates, so call it sparingly.
fn domain_key(host: &str) -> String {
    KEY_CALLS.set(KEY_CALLS.get() + 1);
    host.rsplit('.').collect::<Vec<_>>().join(".")
}

/// Sorts hosts by `domain_key`, so each domain's hosts end up together.
/// Calls `domain_key` exactly once per host.
fn sort_hosts(hosts: &mut [String]) {
    hosts.sort_by_cached_key(|host| domain_key(host));
}

fn main() {
    let mut recent = Recent::new(3);
    for ms in [120, 80, 95, 300] {
        if let Some(dropped) = recent.record(ms) {
            println!("dropped {dropped}");
        }
    }
    println!("average of the last 3: {:?}", recent.average());
    println!("slowest 2: {:?}", slowest(&[120, 80, 95, 300, 210], 2));

    let deploys = BTreeMap::from([(100, "v1".to_string()), (250, "v2".to_string())]);
    println!(
        "deploys before 200: {:?}",
        deploys_between(&deploys, 0, 200)
    );

    let before = HashSet::from(["web-1", "web-2", "db-1"]);
    let after = HashSet::from(["web-1", "web-3", "db-1"]);
    println!("churn: {:?}", churn(&before, &after));

    let mut hosts = ["www.example.org", "api.example.com", "example.com"].map(String::from);
    sort_hosts(&mut hosts);
    println!("{hosts:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_last_few_times() {
        let mut recent = Recent::new(3);
        assert_eq!(recent.average(), None);
        assert_eq!(recent.record(100), None);
        assert_eq!(recent.record(200), None);
        assert_eq!(recent.record(300), None);
        assert_eq!(recent.average(), Some(200.0));
        assert_eq!(recent.record(700), Some(100));
        assert_eq!(recent.record(800), Some(200));
        assert_eq!(recent.average(), Some(600.0));
        assert_eq!(Vec::from(recent.times), [300, 700, 800]);
    }

    #[test]
    fn finds_the_slowest() {
        let times = [120, 80, 95, 300, 210, 95];
        assert_eq!(slowest(&times, 3), [300, 210, 120]);
        assert_eq!(slowest(&times, 10), [300, 210, 120, 95, 95, 80]);
        assert!(slowest(&times, 0).is_empty());
        assert!(slowest(&[], 2).is_empty());
    }

    #[test]
    fn finds_deploys_in_a_window() {
        let deploys = BTreeMap::from([
            (100, "v1".to_string()),
            (250, "v2".to_string()),
            (300, "v3".to_string()),
            (900, "v4".to_string()),
        ]);
        assert_eq!(deploys_between(&deploys, 100, 300), ["v1", "v2"]);
        assert_eq!(deploys_between(&deploys, 101, 901), ["v2", "v3", "v4"]);
        assert!(deploys_between(&deploys, 400, 900).is_empty());
        assert!(deploys_between(&deploys, 300, 300).is_empty());
        assert!(deploys_between(&deploys, 900, 100).is_empty());
    }

    #[test]
    fn finds_churn() {
        let before = HashSet::from(["web-1", "web-2", "db-1", "db-2"]);
        let after = HashSet::from(["web-1", "web-3", "web-4", "db-1"]);
        let (appeared, gone) = churn(&before, &after);
        assert_eq!(appeared, BTreeSet::from(["web-3", "web-4"]));
        assert_eq!(gone, BTreeSet::from(["db-2", "web-2"]));
        let (appeared, gone) = churn(&after, &after);
        assert!(appeared.is_empty() && gone.is_empty());
    }

    #[test]
    fn sorts_hosts_by_domain() {
        let mut hosts = [
            "www.b.org",
            "api.a.com",
            "b.org",
            "mail.a.com",
            "a.com",
            "cdn.b.org",
        ]
        .map(String::from);
        KEY_CALLS.set(0);
        sort_hosts(&mut hosts);
        assert_eq!(
            hosts,
            [
                "a.com",
                "api.a.com",
                "mail.a.com",
                "b.org",
                "cdn.b.org",
                "www.b.org"
            ]
        );
        assert_eq!(KEY_CALLS.get(), 6, "domain_key should run once per host");
    }
}

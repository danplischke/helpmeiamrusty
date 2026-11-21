//! # Checkpoint: a log analyzer (reference solution)
//!
//! - `FromIterator` is where the invariant gets established: collect, then
//!   sort once. `sort_by_key` is stable, so equal times keep their arrival
//!   order (`sort_unstable_by_key` wouldn't promise that). With that impl,
//!   `parse` is a single `filter_map(..).collect()`.
//! - `partition_point(|e| e.at < t)` is a binary search for "the first
//!   entry at or after `t`". Two of them bound the range, and `get(lo..hi)`
//!   returns `None` instead of panicking when `lo > hi`.
//! - A `BTreeMap` iterates in key order, so after a *stable* sort by count
//!   the ties are already alphabetical. `Reverse(count)` sorts descending
//!   without writing a comparator.
//! - `chunk_by` on the error times does the grouping; `first()` and
//!   `last()` with `?` inside `filter_map` avoid `unwrap` (a chunk is never
//!   empty, but the types don't know that).

use std::cmp::Reverse;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
struct Entry {
    /// Seconds since the service started.
    at: u64,
    level: Level,
    service: String,
    message: String,
}

impl Entry {
    /// Parses "<seconds> <LEVEL> <service>: <message>", for example
    /// "17 ERROR db: timeout". `None` if the line doesn't fit that shape.
    fn parse(line: &str) -> Option<Entry> {
        let (at, rest) = line.trim().split_once(' ')?;
        let (level, rest) = rest.split_once(' ')?;
        let (service, message) = rest.split_once(": ")?;
        let level = match level {
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            "ERROR" => Level::Error,
            _ => return None,
        };
        Some(Entry {
            at: at.parse().ok()?,
            level,
            service: service.to_string(),
            message: message.to_string(),
        })
    }
}

/// Log entries sorted by time. Entries with the same time keep the order
/// they arrived in.
#[derive(Debug, Default)]
struct Log {
    entries: Vec<Entry>,
}

impl FromIterator<Entry> for Log {
    /// Collects entries that arrive in any order.
    fn from_iter<I: IntoIterator<Item = Entry>>(iter: I) -> Self {
        let mut entries: Vec<Entry> = iter.into_iter().collect();
        entries.sort_by_key(|e| e.at);
        Log { entries }
    }
}

impl<'a> IntoIterator for &'a Log {
    type Item = &'a Entry;
    type IntoIter = std::slice::Iter<'a, Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl Log {
    /// Parses every line of `text`, skipping the ones that don't parse.
    fn parse(text: &str) -> Log {
        text.lines().filter_map(Entry::parse).collect()
    }

    /// The entries with `from <= at < to`, found by binary search. Empty
    /// when `from >= to`.
    fn between(&self, from: u64, to: u64) -> &[Entry] {
        let lo = self.entries.partition_point(|e| e.at < from);
        let hi = self.entries.partition_point(|e| e.at < to);
        self.entries.get(lo..hi).unwrap_or_default()
    }

    fn errors(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| e.level == Level::Error)
    }

    /// How many errors each service logged, by service name. Services
    /// without errors are left out.
    fn errors_by_service(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for error in self.errors() {
            *counts.entry(error.service.as_str()).or_insert(0) += 1;
        }
        counts
    }

    /// The `k` services with the most errors, as (service, errors), most
    /// first; services with equal counts in alphabetical order.
    fn noisiest(&self, k: usize) -> Vec<(&str, usize)> {
        let mut ranked: Vec<(&str, usize)> = self.errors_by_service().into_iter().collect();
        ranked.sort_by_key(|&(_, count)| Reverse(count));
        ranked.truncate(k);
        ranked
    }

    /// Groups errors into incidents: consecutive errors (ignoring other
    /// levels) less than `gap` seconds apart belong to the same incident.
    /// Returns the time of each incident's first and last error.
    fn incidents(&self, gap: u64) -> Vec<(u64, u64)> {
        let times: Vec<u64> = self.errors().map(|e| e.at).collect();
        times
            .chunk_by(|a, b| b - a < gap)
            .filter_map(|incident| Some((*incident.first()?, *incident.last()?)))
            .collect()
    }
}

const SAMPLE: &str = "\
5 INFO api: started
12 ERROR db: timeout
3 INFO db: started
12 WARN api: slow query
not a log line
14 ERROR db: timeout
15 ERROR api: 502 from upstream
40 ERROR cache: evicted hot key
99 FATAL api: unknown level
41 ERROR db: timeout
";

fn main() {
    let log = Log::parse(SAMPLE);
    for entry in &log {
        println!(
            "{:>3} {:?} {}: {}",
            entry.at, entry.level, entry.service, entry.message
        );
    }
    println!("between 10 and 20: {}", log.between(10, 20).len());
    println!("errors: {:?}", log.errors_by_service());
    println!("noisiest: {:?}", log.noisiest(2));
    println!("incidents: {:?}", log.incidents(10));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn times(entries: &[Entry]) -> Vec<u64> {
        entries.iter().map(|e| e.at).collect()
    }

    fn entry(at: u64, level: Level, service: &str) -> Entry {
        Entry {
            at,
            level,
            service: service.to_string(),
            message: String::new(),
        }
    }

    #[test]
    fn parses_skips_and_sorts() {
        let log = Log::parse(SAMPLE);
        assert_eq!(times(&log.entries), [3, 5, 12, 12, 14, 15, 40, 41]);
        let at_twelve: Vec<Level> = log.entries[2..4].iter().map(|e| e.level).collect();
        assert_eq!(
            at_twelve,
            [Level::Error, Level::Warn],
            "equal times keep their order"
        );
        assert!(Log::parse("").entries.is_empty());
    }

    #[test]
    fn collects_from_any_iterator() {
        let log: Log = [
            entry(9, Level::Info, "api"),
            entry(1, Level::Warn, "db"),
            entry(9, Level::Error, "db"),
        ]
        .into_iter()
        .collect();
        assert_eq!(times(&log.entries), [1, 9, 9]);
        assert_eq!(log.entries[1].level, Level::Info);
    }

    #[test]
    fn iterates_by_reference() {
        let log = Log::parse(SAMPLE);
        let mut services = Vec::new();
        for entry in &log {
            services.push(entry.service.as_str());
        }
        assert_eq!(
            services,
            ["db", "api", "db", "api", "db", "api", "cache", "db"]
        );
    }

    #[test]
    fn finds_entries_between_two_times() {
        let log = Log::parse(SAMPLE);
        assert_eq!(times(log.between(12, 15)), [12, 12, 14]);
        assert_eq!(times(log.between(13, 41)), [14, 15, 40]);
        assert_eq!(times(log.between(41, 1000)), [41]);
        assert!(log.between(0, 3).is_empty());
        assert!(log.between(20, 10).is_empty());
        assert!(Log::default().between(0, 10).is_empty());
    }

    #[test]
    fn counts_errors_by_service() {
        let log = Log::parse(SAMPLE);
        assert_eq!(
            log.errors_by_service(),
            BTreeMap::from([("api", 1), ("cache", 1), ("db", 3)])
        );
        assert!(Log::default().errors_by_service().is_empty());
    }

    #[test]
    fn ranks_the_noisiest_services() {
        let log = Log::parse(SAMPLE);
        assert_eq!(log.noisiest(2), [("db", 3), ("api", 1)]);
        assert_eq!(log.noisiest(10), [("db", 3), ("api", 1), ("cache", 1)]);
        assert!(log.noisiest(0).is_empty());
    }

    #[test]
    fn groups_errors_into_incidents() {
        let log = Log::parse(SAMPLE);
        assert_eq!(log.incidents(5), [(12, 15), (40, 41)]);
        assert_eq!(log.incidents(30), [(12, 41)]);
        assert_eq!(
            log.incidents(1),
            [(12, 12), (14, 14), (15, 15), (40, 40), (41, 41)]
        );
        assert!(Log::default().incidents(5).is_empty());
    }
}

//! # Sorting and searching
//!
//! Race results. Each function is a sensible first attempt that the
//! compiler rejects, for a reason worth knowing: `f64` isn't `Ord`, the key
//! closure of a `_by_key` method that takes `&mut self` can't return a
//! borrow of the element, and a binary search returns more than an index.
//! Make each one compile without changing what it's meant to do.

#[derive(Debug, Clone, PartialEq)]
struct Finish {
    runner: String,
    club: String,
    secs: f64,
}

/// Sorts by runner name, A to Z.
fn sort_by_runner(results: &mut [Finish]) {
    results.sort_by_key(|f| &f.runner);
}

/// The fastest finisher; on a tie, the one listed first. `None` if empty.
fn fastest(results: &[Finish]) -> Option<&Finish> {
    results.iter().min_by_key(|f| f.secs)
}

/// Inserts `new` into `sorted`, which is sorted by runner name, at the
/// position that keeps it sorted. Uses binary search to find the spot.
fn insert_sorted(sorted: &mut Vec<Finish>, new: Finish) {
    let pos = sorted.binary_search_by(|f| f.runner.cmp(&new.runner));
    sorted.insert(pos, new);
}

/// The first finisher of each club. `results` are sorted by club, and
/// within a club by time, so each club's winner comes first in its group.
fn club_winners(mut results: Vec<Finish>) -> Vec<Finish> {
    results.dedup_by_key(|f| &f.club);
    results
}

fn finish(runner: &str, club: &str, secs: f64) -> Finish {
    Finish {
        runner: runner.to_string(),
        club: club.to_string(),
        secs,
    }
}

fn main() {
    let mut results = vec![
        finish("Kofi", "Harriers", 1520.5),
        finish("Ada", "Striders", 1498.0),
        finish("Mei", "Harriers", 1601.2),
    ];
    println!("fastest: {:?}", fastest(&results).map(|f| &f.runner));
    sort_by_runner(&mut results);
    insert_sorted(&mut results, finish("Bea", "Striders", 1498.0));
    println!("by runner: {results:?}");
    let by_club = vec![
        finish("Kofi", "Harriers", 1520.5),
        finish("Mei", "Harriers", 1601.2),
        finish("Ada", "Striders", 1498.0),
    ];
    println!("winners: {:?}", club_winners(by_club));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> Vec<Finish> {
        vec![
            finish("Mei", "Harriers", 1601.2),
            finish("Ada", "Striders", 1498.0),
            finish("Kofi", "Harriers", 1520.5),
            finish("Bea", "Striders", 1498.0),
        ]
    }

    fn runners(results: &[Finish]) -> Vec<&str> {
        results.iter().map(|f| f.runner.as_str()).collect()
    }

    #[test]
    fn sorts_by_runner() {
        let mut results = field();
        sort_by_runner(&mut results);
        assert_eq!(runners(&results), ["Ada", "Bea", "Kofi", "Mei"]);
    }

    #[test]
    fn finds_the_fastest() {
        let results = field();
        assert_eq!(fastest(&results).map(|f| f.runner.as_str()), Some("Ada"));
        assert_eq!(
            fastest(&results[2..]).map(|f| f.runner.as_str()),
            Some("Bea")
        );
        assert_eq!(fastest(&[]), None);
    }

    #[test]
    fn inserts_in_order() {
        let mut sorted = vec![
            finish("Ada", "Striders", 1498.0),
            finish("Kofi", "Harriers", 1520.5),
            finish("Mei", "Harriers", 1601.2),
        ];
        insert_sorted(&mut sorted, finish("Cy", "Harriers", 1555.0));
        insert_sorted(&mut sorted, finish("Zoe", "Striders", 1490.0));
        insert_sorted(&mut sorted, finish("Abe", "Tri Club", 1702.3));
        assert_eq!(runners(&sorted), ["Abe", "Ada", "Cy", "Kofi", "Mei", "Zoe"]);
        insert_sorted(&mut sorted, finish("Kofi", "Tri Club", 1800.0));
        assert_eq!(
            runners(&sorted),
            ["Abe", "Ada", "Cy", "Kofi", "Kofi", "Mei", "Zoe"]
        );
        let mut empty = Vec::new();
        insert_sorted(&mut empty, finish("Ada", "Striders", 1498.0));
        assert_eq!(runners(&empty), ["Ada"]);
    }

    #[test]
    fn keeps_each_clubs_winner() {
        let sorted = vec![
            finish("Kofi", "Harriers", 1520.5),
            finish("Mei", "Harriers", 1601.2),
            finish("Ada", "Striders", 1498.0),
            finish("Bea", "Striders", 1498.0),
            finish("Lou", "Tri Club", 1700.0),
        ];
        assert_eq!(runners(&club_winners(sorted)), ["Kofi", "Ada", "Lou"]);
        assert!(club_winners(Vec::new()).is_empty());
    }
}

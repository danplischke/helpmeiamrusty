//! # Collecting into the right shape (reference solution)
//!
//! - `String` implements `FromIterator` for `char`, `&str` and `String`, so
//!   pieces can be collected straight into one string. Use `join` when you
//!   need a separator *between* items.
//! - Collecting `(key, value)` pairs into a map keeps the **last** value
//!   for a repeated key, because each one is an `insert`. Ordered input
//!   makes that "the latest wins".
//! - `partition` splits by a predicate and keeps the items as they are;
//!   `unzip` splits pairs into two collections. Both walk the input once.
//! - Implement `Extend` first and `FromIterator` becomes
//!   `let mut h = Self::default(); h.extend(iter); h`. Accepting any
//!   `IntoIterator` lets callers pass a Vec, an array or a chain.
//! - `std::array::from_fn(|i| ..)` builds a fixed-size array from its
//!   indices: no `Default` or `Copy` needed, and no placeholder values.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct Score {
    student: String,
    points: u32,
}

/// One line per score, "<student>: <points>\n", in order.
fn report(scores: &[Score]) -> String {
    scores
        .iter()
        .map(|s| format!("{}: {}\n", s.student, s.points))
        .collect()
}

/// Each student's most recent score. `scores` are in the order they were
/// recorded.
fn latest(scores: &[Score]) -> HashMap<&str, u32> {
    scores
        .iter()
        .map(|s| (s.student.as_str(), s.points))
        .collect()
}

/// The scores that reached `pass_mark` and the ones that didn't, each in
/// their original order.
fn pass_fail(scores: &[Score], pass_mark: u32) -> (Vec<&Score>, Vec<&Score>) {
    scores.iter().partition(|s| s.points >= pass_mark)
}

/// Two parallel columns for a chart: the names and the points.
fn columns(scores: &[Score]) -> (Vec<&str>, Vec<u32>) {
    scores
        .iter()
        .map(|s| (s.student.as_str(), s.points))
        .unzip()
}

/// How many scores fell in each band of ten points: 0-9, 10-19, ...,
/// 90-99, and finally 100 (anything over 100 counts as 100).
#[derive(Debug, Default, PartialEq)]
struct Histogram {
    bands: [u32; 11],
}

impl Histogram {
    /// The band labels: "0-9", "10-19", ..., "90-99", "100".
    fn labels() -> [String; 11] {
        std::array::from_fn(|i| match i {
            10 => "100".to_string(),
            _ => format!("{}-{}", i * 10, i * 10 + 9),
        })
    }
}

impl FromIterator<u32> for Histogram {
    fn from_iter<I: IntoIterator<Item = u32>>(iter: I) -> Self {
        let mut histogram = Histogram::default();
        histogram.extend(iter);
        histogram
    }
}

impl Extend<u32> for Histogram {
    fn extend<I: IntoIterator<Item = u32>>(&mut self, iter: I) {
        for points in iter {
            self.bands[points.min(100) as usize / 10] += 1;
        }
    }
}

fn score(student: &str, points: u32) -> Score {
    Score {
        student: student.to_string(),
        points,
    }
}

fn main() {
    let scores = vec![score("Ana", 72), score("Ben", 48), score("Ana", 91)];
    print!("{}", report(&scores));
    println!("latest: {:?}", latest(&scores).get("Ana"));
    let (passed, failed) = pass_fail(&scores, 50);
    println!("{} passed, {} failed", passed.len(), failed.len());
    println!("{:?}", columns(&scores));
    let histogram: Histogram = scores.iter().map(|s| s.points).collect();
    for (label, count) in Histogram::labels().iter().zip(histogram.bands) {
        if count > 0 {
            println!("{label:>6}: {count}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores() -> Vec<Score> {
        vec![
            score("Ana", 72),
            score("Ben", 48),
            score("Cleo", 100),
            score("Ana", 91),
            score("Ben", 55),
        ]
    }

    #[test]
    fn reports_one_line_per_score() {
        assert_eq!(report(&scores()[..3]), "Ana: 72\nBen: 48\nCleo: 100\n");
        assert_eq!(report(&[]), "");
    }

    #[test]
    fn keeps_the_latest_score() {
        let scores = scores();
        assert_eq!(
            latest(&scores),
            HashMap::from([("Ana", 91), ("Ben", 55), ("Cleo", 100)])
        );
    }

    #[test]
    fn splits_pass_and_fail() {
        let scores = scores();
        let (passed, failed) = pass_fail(&scores, 55);
        assert_eq!(passed, [&scores[0], &scores[2], &scores[3], &scores[4]]);
        assert_eq!(failed, [&scores[1]]);
        let (passed, failed) = pass_fail(&scores, 101);
        assert!(passed.is_empty());
        assert_eq!(failed.len(), 5);
    }

    #[test]
    fn splits_into_columns() {
        let scores = scores();
        let (names, points) = columns(&scores[..2]);
        assert_eq!(names, ["Ana", "Ben"]);
        assert_eq!(points, [72, 48]);
    }

    #[test]
    fn labels_the_bands() {
        let labels = Histogram::labels();
        assert_eq!(labels[0], "0-9");
        assert_eq!(labels[5], "50-59");
        assert_eq!(labels[9], "90-99");
        assert_eq!(labels[10], "100");
    }

    #[test]
    fn collects_into_a_histogram() {
        let histogram: Histogram = scores().iter().map(|s| s.points).collect();
        assert_eq!(histogram.bands, [0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1]);
        let empty: Histogram = std::iter::empty().collect();
        assert_eq!(empty, Histogram::default());
    }

    #[test]
    fn extends_a_histogram() {
        let mut histogram: Histogram = [5, 15].into_iter().collect();
        histogram.extend(vec![9, 100, 250]);
        histogram.extend((0..3).map(|i| i * 10));
        assert_eq!(histogram.bands, [3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 2]);
    }
}

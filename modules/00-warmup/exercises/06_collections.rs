//! # Vec, HashMap, BTreeMap and the entry API
//!
//! The bread and butter of everyday Rust. Implement the functions; the
//! `entry` API, `retain`, `sort_by` and `dedup` will all come in handy.

use std::collections::{BTreeMap, HashMap};

/// Counts words case-insensitively. A word is a run of letters, digits and
/// apostrophes: "Don't stop, don't!" has "don't" twice and "stop" once.
fn word_counts(text: &str) -> HashMap<String, usize> {
    todo!("count the words in {text:?}")
}

/// The `n` most frequent words, most frequent first, ties in alphabetical
/// order. The `&str`s borrow from the map's keys, so nothing is copied.
fn top_words(counts: &HashMap<String, usize>, n: usize) -> Vec<(&str, usize)> {
    todo!("the top {n} of {counts:?}")
}

/// Removes, in place, every word shorter than `min_len` *characters*.
fn drop_short(words: &mut Vec<String>, min_len: usize) {
    todo!("drop words shorter than {min_len} from {words:?}")
}

/// Lowercased words grouped by their first letter. Each group is sorted
/// and has no duplicates.
fn index_by_letter(words: &[&str]) -> BTreeMap<char, Vec<String>> {
    todo!("index {words:?}")
}

fn main() {
    let text = "the cat and the hat and the bat sat on the mat";
    let counts = word_counts(text);
    println!("top 3: {:?}", top_words(&counts, 3));
    let mut words: Vec<String> = text.split(' ').map(String::from).collect();
    drop_short(&mut words, 3);
    println!("{} words of 3+ letters", words.len());
    println!(
        "{:?}",
        index_by_letter(&["Banana", "apple", "blueberry", "Apple"])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words() {
        let counts = word_counts("Don't stop, don't! STOP? go");
        assert_eq!(counts.len(), 3);
        assert_eq!(counts["don't"], 2);
        assert_eq!(counts["stop"], 2);
        assert_eq!(counts["go"], 1);
        assert!(word_counts("  ...  ").is_empty());
    }

    #[test]
    fn ranks_top_words() {
        let counts: HashMap<String, usize> = [("b", 3), ("a", 3), ("c", 1), ("d", 5)]
            .into_iter()
            .map(|(w, c)| (w.to_string(), c))
            .collect();
        assert_eq!(top_words(&counts, 3), [("d", 5), ("a", 3), ("b", 3)]);
        assert_eq!(top_words(&counts, 10).len(), 4);
        assert!(top_words(&counts, 0).is_empty());
    }

    #[test]
    fn drops_short_words_by_chars() {
        let mut words: Vec<String> = ["a", "abc", "héé", "ab"].map(String::from).to_vec();
        drop_short(&mut words, 3);
        assert_eq!(words, ["abc", "héé"]);
    }

    #[test]
    fn indexes_by_first_letter() {
        let index = index_by_letter(&[
            "Banana",
            "apple",
            "blueberry",
            "avocado",
            "banana",
            "cherry",
        ]);
        let expected: BTreeMap<char, Vec<String>> = [
            ('a', vec!["apple", "avocado"]),
            ('b', vec!["banana", "blueberry"]),
            ('c', vec!["cherry"]),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().map(String::from).collect()))
        .collect();
        assert_eq!(index, expected);
    }
}

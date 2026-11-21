//! # When one lifetime isn't enough
//!
//! A single lifetime parameter on two references says "these live equally
//! long". Both structs here hold two references that have nothing to do
//! with each other, and the tests notice. Give each struct a second
//! lifetime parameter and use each one where it belongs. The bodies are
//! fine as they are.

use std::str::SplitWhitespace;

/// The words of a text that contain a needle, in order.
struct WordsContaining<'a> {
    words: SplitWhitespace<'a>,
    needle: &'a str,
}

/// Iterates over the whitespace-separated words of `haystack` that contain
/// `needle`. The words are slices of `haystack`. An empty needle matches
/// every word.
fn words_containing<'a>(haystack: &'a str, needle: &'a str) -> WordsContaining<'a> {
    WordsContaining {
        words: haystack.split_whitespace(),
        needle,
    }
}

impl<'a> Iterator for WordsContaining<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let needle = self.needle;
        self.words.find(|word| word.contains(needle))
    }
}

/// Appends the fields of lines to a caller-provided Vec, so one buffer can
/// be reused for many batches of lines.
struct FieldSink<'a> {
    out: &'a mut Vec<&'a str>,
    separator: char,
}

impl<'a> FieldSink<'a> {
    fn new(out: &'a mut Vec<&'a str>, separator: char) -> Self {
        FieldSink { out, separator }
    }

    /// Appends the trimmed fields of `line` and returns how many it added.
    fn push_line(&mut self, line: &'a str) -> usize {
        let before = self.out.len();
        self.out.extend(line.split(self.separator).map(str::trim));
        self.out.len() - before
    }
}

fn main() {
    for word in words_containing("the borrow checker borrows", "borrow") {
        print!("{word} ");
    }
    println!();

    let text = String::from("lexer, ann\nparser, bob");
    let mut fields = Vec::new();
    {
        let mut sink = FieldSink::new(&mut fields, ',');
        for line in text.lines() {
            sink.push_line(line);
        }
    }
    println!("{fields:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_words() {
        let found: Vec<&str> = words_containing("borrowing is not sorrowful", "rrow").collect();
        assert_eq!(found, ["borrowing", "sorrowful"]);
        assert_eq!(words_containing("abc", "x").next(), None);
        assert_eq!(words_containing(" a  b ", "").count(), 2);
    }

    #[test]
    fn words_outlive_the_needle() {
        let text = String::from("the borrow checker borrows");
        let found: Vec<&str>;
        {
            let needle = String::from("borrow");
            found = words_containing(&text, &needle).collect();
        }
        assert_eq!(found, ["borrow", "borrows"]);
    }

    #[test]
    fn counts_fields() {
        let mut fields = Vec::new();
        let mut sink = FieldSink::new(&mut fields, ';');
        assert_eq!(sink.push_line("a; b ;c"), 3);
        assert_eq!(sink.push_line(""), 1);
    }

    #[test]
    fn the_buffer_is_free_once_the_sink_is_gone() {
        let text = String::from("a, b\nc");
        let mut fields = Vec::new();
        {
            let mut sink = FieldSink::new(&mut fields, ',');
            for line in text.lines() {
                sink.push_line(line);
            }
        }
        assert_eq!(fields, ["a", "b", "c"]);
        fields.clear();
        FieldSink::new(&mut fields, ',').push_line("d,e");
        assert_eq!(fields, ["d", "e"]);
    }
}

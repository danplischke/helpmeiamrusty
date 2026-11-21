//! # When one lifetime isn't enough (reference solution)
//!
//! - With one `'a` for haystack and needle, the compiler has to pick a
//!   lifetime both satisfy: the shorter one. `&'long str` happily shrinks
//!   to `&'short str` (shared references are *covariant*), so it compiles,
//!   but every `&'a str` that comes out is now limited by the needle.
//! - `WordsContaining<'h, 'n>` says what's true: the words borrow from the
//!   haystack (`Item = &'h str`); the needle only has to last while you
//!   iterate. The constructor needs both names, since elision can't pick
//!   between two inputs.
//! - `&'a mut Vec<&'a str>` is the classic trap. `&mut T` is *invariant*
//!   in `T`, so `'a` can't shrink: it's pinned to the text's lifetime, and
//!   the same `'a` then keeps the Vec mutably borrowed for that long too.
//!   `&'v mut Vec<&'s str>` lets the borrow end with the sink.
//! - The symptom to recognise: "borrowed as mutable ... later used here",
//!   pointing at a borrow you thought had ended. Look for one lifetime
//!   doing two jobs.

use std::str::SplitWhitespace;

/// The words of a text that contain a needle, in order.
struct WordsContaining<'h, 'n> {
    words: SplitWhitespace<'h>,
    needle: &'n str,
}

/// Iterates over the whitespace-separated words of `haystack` that contain
/// `needle`. The words are slices of `haystack`. An empty needle matches
/// every word.
fn words_containing<'h, 'n>(haystack: &'h str, needle: &'n str) -> WordsContaining<'h, 'n> {
    WordsContaining {
        words: haystack.split_whitespace(),
        needle,
    }
}

impl<'h> Iterator for WordsContaining<'h, '_> {
    type Item = &'h str;

    fn next(&mut self) -> Option<&'h str> {
        let needle = self.needle;
        self.words.find(|word| word.contains(needle))
    }
}

/// Appends the fields of lines to a caller-provided Vec, so one buffer can
/// be reused for many batches of lines.
struct FieldSink<'v, 's> {
    out: &'v mut Vec<&'s str>,
    separator: char,
}

impl<'v, 's> FieldSink<'v, 's> {
    fn new(out: &'v mut Vec<&'s str>, separator: char) -> Self {
        FieldSink { out, separator }
    }

    /// Appends the trimmed fields of `line` and returns how many it added.
    fn push_line(&mut self, line: &'s str) -> usize {
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

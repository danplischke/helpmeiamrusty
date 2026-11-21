//! # Let elision write it (reference solution)
//!
//! - Rule 1: each elided input reference gets its own lifetime. That's why
//!   `article: &str` and `term: &str` need no name: nothing returned
//!   borrows from them.
//! - Rule 2: with exactly one input lifetime, every elided output lifetime
//!   is that one, tuples and `Option`s included (`split_entry`,
//!   `first_sentence`). Rule 3: with `&self` or `&mut self`, outputs get
//!   `self`'s lifetime.
//! - Keep what elision can't say: `shorter` may return either argument;
//!   `strip_article` returns from `text` only; `define` returns from the
//!   *data* (`'a`), not from `&self`; `label` returns `'static` literals.
//!   Elision would tie each of those to the wrong (or a shorter) borrow.
//! - `const` and `static` items are `'static` already. In a type path,
//!   write `Entry<'_>` rather than bare `Entry`, so the borrow stays
//!   visible (rustc warns `mismatched_lifetime_syntaxes` if you hide it).

/// Shown for terms that aren't in the glossary.
const NOT_FOUND: &str = "(not in glossary)";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Noun,
    Verb,
}

impl Kind {
    /// "noun" or "verb".
    fn label(&self) -> &'static str {
        match self {
            Kind::Noun => "noun",
            Kind::Verb => "verb",
        }
    }
}

/// One glossary line: "<term> (n): <definition>" or "<term> (v): <definition>".
#[derive(Debug, PartialEq)]
struct Entry<'a> {
    term: &'a str,
    kind: Kind,
    definition: &'a str,
}

/// Splits a line at its first colon and trims both halves.
fn split_entry(line: &str) -> Option<(&str, &str)> {
    let (head, definition) = line.split_once(':')?;
    Some((head.trim(), definition.trim()))
}

/// Parses one line. `None` if it doesn't follow the format.
fn parse_entry(line: &str) -> Option<Entry<'_>> {
    let (head, definition) = split_entry(line)?;
    let (term, kind) = head.rsplit_once(' ')?;
    let kind = match kind {
        "(n)" => Kind::Noun,
        "(v)" => Kind::Verb,
        _ => return None,
    };
    Some(Entry {
        term: term.trim(),
        kind,
        definition,
    })
}

/// Everything before the first ". ", or all of `text` if there is none.
fn first_sentence(text: &str) -> &str {
    text.split_once(". ").map_or(text, |(first, _)| first)
}

/// The number of words in `text`.
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// The shorter of two definitions; `a` on ties.
fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str {
    if b.len() < a.len() { b } else { a }
}

/// `text` without a leading `article` and the space after it. Unchanged if
/// it doesn't start with "<article> ".
fn strip_article<'t>(text: &'t str, article: &str) -> &'t str {
    text.strip_prefix(article)
        .and_then(|rest| rest.strip_prefix(' '))
        .unwrap_or(text)
}

struct Glossary<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> Glossary<'a> {
    /// One entry per line; lines that don't parse are skipped.
    fn parse(text: &'a str) -> Self {
        Glossary {
            entries: text.lines().filter_map(parse_entry).collect(),
        }
    }

    /// Whether `term` has an entry.
    fn is_defined(&self, term: &str) -> bool {
        self.entries.iter().any(|entry| entry.term == term)
    }

    /// The definition of `term`, or `NOT_FOUND`.
    fn define(&self, term: &str) -> &'a str {
        self.entries
            .iter()
            .find(|entry| entry.term == term)
            .map_or(NOT_FOUND, |entry| entry.definition)
    }

    /// The entry with the most words in its definition; the first on ties.
    fn most_verbose(&self) -> Option<&Entry<'a>> {
        let mut best: Option<&Entry> = None;
        for entry in &self.entries {
            if best.is_none_or(|b| word_count(entry.definition) > word_count(b.definition)) {
                best = Some(entry);
            }
        }
        best
    }
}

const GLOSSARY: &str = "\
borrow (v): to take a reference without taking ownership
lifetime (n): the region of code where a reference is valid. Usually inferred.
this line has no colon
elision (n): the compiler filling in lifetimes you left out
outlive (v): to stay valid for longer than";

fn main() {
    let glossary = Glossary::parse(GLOSSARY);
    println!("{}", first_sentence(glossary.define("lifetime")));
    println!("{}", glossary.define("variance"));
    if let Some(entry) = glossary.most_verbose() {
        println!("{} ({})", entry.term, entry.kind.label());
    }
    println!("{}", strip_article("the borrow checker", "the"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entries() {
        assert_eq!(
            parse_entry("move (v): to transfer ownership"),
            Some(Entry {
                term: "move",
                kind: Kind::Verb,
                definition: "to transfer ownership",
            })
        );
        assert_eq!(parse_entry("move: no kind"), None);
        assert_eq!(parse_entry("move (adj): wrong kind"), None);
        assert_eq!(split_entry(" a : b: c "), Some(("a", "b: c")));
        assert_eq!(Glossary::parse(GLOSSARY).entries.len(), 4);
    }

    #[test]
    fn helpers() {
        assert_eq!(first_sentence("One. Two. Three."), "One");
        assert_eq!(first_sentence("No full stop"), "No full stop");
        assert_eq!(word_count("  three  short words "), 3);
        assert_eq!(shorter("abc", "de"), "de");
        assert_eq!(shorter("ab", "cd"), "ab");
        assert_eq!(strip_article("an owner", "an"), "owner");
        assert_eq!(strip_article("another", "an"), "another");
    }

    #[test]
    fn stripped_text_outlives_the_article() {
        let text = String::from("the stack");
        let article = String::from("the");
        let rest = strip_article(&text, &article);
        drop(article);
        assert_eq!(rest, "stack");
    }

    #[test]
    fn definitions_outlive_the_glossary() {
        let text = String::from(GLOSSARY);
        let definition = {
            let glossary = Glossary::parse(&text);
            glossary.define("borrow")
        };
        assert_eq!(definition, "to take a reference without taking ownership");
        assert_eq!(Glossary::parse(&text).define("variance"), NOT_FOUND);
    }

    #[test]
    fn labels_outlive_the_entries() {
        let labels: Vec<&str> = {
            let glossary = Glossary::parse(GLOSSARY);
            glossary.entries.iter().map(|e| e.kind.label()).collect()
        };
        assert_eq!(labels, ["verb", "noun", "noun", "verb"]);
    }

    #[test]
    fn lookups() {
        let glossary = Glossary::parse(GLOSSARY);
        assert!(glossary.is_defined("elision"));
        assert!(!glossary.is_defined("Elision"));
        assert_eq!(glossary.most_verbose().map(|e| e.term), Some("lifetime"));
        let tie = Glossary::parse("a (n): one two\nb (n): three four");
        assert_eq!(tie.most_verbose().map(|e| e.term), Some("a"));
        assert_eq!(Glossary::parse("").most_verbose(), None);
    }
}

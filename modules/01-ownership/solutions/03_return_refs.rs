//! # Returning references (reference solution)
//!
//! - `longest_word` never needed a `String`: `best` can be a `&str` into
//!   `text` from the start. With one reference parameter, lifetime elision
//!   ties the output to it automatically.
//! - With *two* reference parameters, elision gives up (E0106). Name the
//!   relationship: the result borrows from `haystack` only. Putting `'a` on
//!   `needle` too would compile here, but would make callers keep `needle`
//!   alive as long as the result, and the test drops it early.
//! - `make_greeting` builds a new string, so it has to return it owned.
//!   No lifetime annotation can make a local live longer.

/// The longest word (the first one on ties), as a slice of `text`: no copying.
fn longest_word(text: &str) -> &str {
    let mut best = "";
    for word in text.split_whitespace() {
        if word.len() > best.len() {
            best = word;
        }
    }
    best
}

/// The first line of `haystack` that contains `needle`.
fn first_match<'a>(haystack: &'a str, needle: &str) -> Option<&'a str> {
    haystack.lines().find(|line| line.contains(needle))
}

/// "Hello, <name>!"
fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

fn main() {
    let text = "the borrow checker is your pair programmer";
    println!("{}", longest_word(text));
    println!("{:?}", first_match("alpha\nbeta\ngamma", "mm"));
    println!("{}", make_greeting("Ferris"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_word_borrows_from_input() {
        let text = String::from("a quick brownish fox");
        let word = longest_word(&text);
        assert_eq!(word, "brownish");
        let start = text.as_ptr() as usize;
        let inside = start..start + text.len();
        assert!(
            inside.contains(&(word.as_ptr() as usize)),
            "should point into `text`, not into a new String"
        );
    }

    #[test]
    fn longest_word_edge_cases() {
        assert_eq!(longest_word("ab cd ef"), "ab");
        assert_eq!(longest_word(""), "");
    }

    #[test]
    fn match_outlives_needle() {
        let text = String::from("alpha\nbeta\ngamma");
        let found;
        {
            let needle = String::from("et");
            found = first_match(&text, &needle);
        } // `needle` is dropped here; `found` may only borrow from `text`
        assert_eq!(found, Some("beta"));
        assert_eq!(first_match(&text, "zeta"), None);
    }

    #[test]
    fn greeting_is_owned() {
        let greeting = make_greeting("Ferris");
        assert_eq!(greeting, "Hello, Ferris!");
    }
}

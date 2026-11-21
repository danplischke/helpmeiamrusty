//! # Returning references
//!
//! A function can only return a reference to something that outlives the
//! call: data borrowed from its arguments, or `'static` data. Fix these so
//! they compile, choosing between "borrow from the input" and "return an
//! owned value" for each one.

/// The longest word (the first one on ties), as a slice of `text`: no copying.
fn longest_word(text: &str) -> &str {
    let mut best = String::new();
    for word in text.split_whitespace() {
        if word.len() > best.len() {
            best = word.to_string();
        }
    }
    &best
}

/// The first line of `haystack` that contains `needle`.
fn first_match(haystack: &str, needle: &str) -> Option<&str> {
    haystack.lines().find(|line| line.contains(needle))
}

/// "Hello, <name>!"
fn make_greeting(name: &str) -> &str {
    let greeting = format!("Hello, {name}!");
    &greeting
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

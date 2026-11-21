//! # Strings: `String`, `&str`, chars and bytes (reference solution)
//!
//! - `split` takes a closure as the pattern, so one call splits on whitespace
//!   *and* hyphens. Empty pieces have no first char, so `filter_map` skips them.
//! - `char::to_uppercase` returns an iterator, not a `char`: some characters
//!   uppercase to several ('ß' -> "SS"). `flat_map` handles that.
//! - `char_indices().nth(n)` finds the *byte* offset of the n-th character,
//!   which makes `&s[..offset]` safe to slice.
//! - `filter` and `flat_map` keep the iterator double-ended, so it can be
//!   compared with its own reverse without collecting into a String.

/// First letter of each word, uppercased. Words are separated by whitespace
/// or hyphens: "complementary metal-oxide semiconductor" -> "CMOS".
fn acronym(phrase: &str) -> String {
    phrase
        .split(|c: char| c.is_whitespace() || c == '-')
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

/// Uppercases the first letter of each space-separated word and lowercases
/// the rest: "hELLO wORLD" -> "Hello World".
fn title_case(s: &str) -> String {
    s.split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(char::to_lowercase))
                    .collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// The first `n` characters (not bytes) of `s`, borrowed from `s`.
/// If `s` is shorter than that, all of it.
fn first_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((byte_offset, _)) => &s[..byte_offset],
        None => s,
    }
}

/// True if `s` reads the same backwards, ignoring case and anything that
/// isn't a letter or digit: "A man, a plan, a canal: Panama!" is one.
fn is_palindrome(s: &str) -> bool {
    let cleaned = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase);
    cleaned.clone().eq(cleaned.rev())
}

fn main() {
    println!("{}", acronym("Portable Network Graphics"));
    println!("{}", title_case("the rUST programming LANGUAGE"));
    println!("{}", first_chars("Grüße aus Köln", 5));
    println!("{}", is_palindrome("Was it a car or a cat I saw?"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_acronyms() {
        assert_eq!(acronym("Portable Network Graphics"), "PNG");
        assert_eq!(acronym("complementary metal-oxide semiconductor"), "CMOS");
        assert_eq!(acronym("  leading   spaces "), "LS");
        assert_eq!(acronym(""), "");
    }

    #[test]
    fn title_cases() {
        assert_eq!(title_case("hELLO wORLD"), "Hello World");
        assert_eq!(title_case("émile zola"), "Émile Zola");
        assert_eq!(title_case("a"), "A");
        assert_eq!(title_case(""), "");
    }

    #[test]
    fn counts_chars_not_bytes() {
        assert_eq!(first_chars("héllo", 2), "hé");
        assert_eq!(first_chars("héllo", 10), "héllo");
        assert_eq!(first_chars("", 3), "");
        let s = String::from("borrowed");
        assert_eq!(
            first_chars(&s, 3).as_ptr(),
            s.as_ptr(),
            "must borrow from `s`"
        );
    }

    #[test]
    fn detects_palindromes() {
        assert!(is_palindrome("A man, a plan, a canal: Panama!"));
        assert!(is_palindrome("été"));
        assert!(is_palindrome(""));
        assert!(!is_palindrome("Ferris"));
    }
}

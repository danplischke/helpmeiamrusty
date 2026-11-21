//! # Strings: `String`, `&str`, chars and bytes
//!
//! A `String` owns UTF-8 bytes; a `&str` borrows some. `len()` counts bytes,
//! not characters, and `&s[a..b]` slices by *byte* offset (it panics if an
//! offset lands inside a multi-byte character). Implement the functions.

/// First letter of each word, uppercased. Words are separated by whitespace
/// or hyphens: "complementary metal-oxide semiconductor" -> "CMOS".
fn acronym(phrase: &str) -> String {
    todo!("acronym of {phrase:?}")
}

/// Uppercases the first letter of each space-separated word and lowercases
/// the rest: "hELLO wORLD" -> "Hello World".
fn title_case(s: &str) -> String {
    todo!("title case of {s:?}")
}

/// The first `n` characters (not bytes) of `s`, borrowed from `s`.
/// If `s` is shorter than that, all of it.
fn first_chars(s: &str, n: usize) -> &str {
    todo!("first {n} chars of {s:?}")
}

/// True if `s` reads the same backwards, ignoring case and anything that
/// isn't a letter or digit: "A man, a plan, a canal: Panama!" is one.
fn is_palindrome(s: &str) -> bool {
    todo!("is {s:?} a palindrome?")
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

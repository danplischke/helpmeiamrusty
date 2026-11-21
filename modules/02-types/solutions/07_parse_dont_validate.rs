//! # Parse, don't validate (reference solution)
//!
//! - A validator returns `bool` and the caller has to remember what it
//!   proved. A parser returns a new type, and the proof travels with the
//!   value: `welcome` and `same_domain` can't receive bad input.
//! - The private field is what makes it a guarantee. Outside `account`,
//!   `Username(String::from("??"))` is E0603 "tuple struct constructor is
//!   private", and there's no `&mut String` to edit it through. Hand out
//!   `&str`, never `&mut`.
//! - Normalize while parsing (trim, lowercase the domain), so equality and
//!   comparisons downstream just work, and keep what parsing learned
//!   (`at`) so accessors don't redo it.
//! - Idiomatic extras: `impl FromStr` (so `"ferris".parse::<Username>()`
//!   works), `TryFrom<String>`, `AsRef<str>`, `Display`. Serde users can
//!   route deserialization through the parser with `#[serde(try_from = "String")]`.
//! - The standard library works the same way: `String` is bytes proven to
//!   be UTF-8 once, by `String::from_utf8`; `NonZeroU32::new` returns an
//!   `Option`.

mod account {
    /// A username: 3 to 16 characters, all ASCII lowercase letters, digits
    /// or `_`, starting with a letter. Stored normalized.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Username(String);

    #[derive(Debug, PartialEq)]
    pub enum UsernameError {
        /// Too short or too long. Holds the length in characters.
        Length(usize),
        /// Holds the first character that isn't allowed.
        InvalidChar(char),
        /// Starts with a digit or `_`.
        BadStart,
    }

    impl Username {
        /// Trims surrounding whitespace and lowercases ASCII letters, then
        /// checks the rules in order: length, characters, first character.
        pub fn parse(raw: &str) -> Result<Self, UsernameError> {
            let name = raw.trim().to_ascii_lowercase();
            let len = name.chars().count();
            if !(3..=16).contains(&len) {
                return Err(UsernameError::Length(len));
            }
            if let Some(bad) = name
                .chars()
                .find(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_'))
            {
                return Err(UsernameError::InvalidChar(bad));
            }
            if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
                return Err(UsernameError::BadStart);
            }
            Ok(Username(name))
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    /// An email address: a non-empty local part, one `@`, and a domain that
    /// contains a `.` but doesn't start or end with one. Surrounding
    /// whitespace is trimmed and the domain is lowercased; the local part
    /// keeps its case.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Email {
        address: String,
        /// The byte offset of the `@` in `address`.
        at: usize,
    }

    impl Email {
        pub fn parse(raw: &str) -> Option<Self> {
            let (local, domain) = raw.trim().split_once('@')?;
            let valid = !local.is_empty()
                && !domain.contains('@')
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.');
            valid.then(|| Email {
                address: format!("{local}@{}", domain.to_ascii_lowercase()),
                at: local.len(),
            })
        }

        pub fn as_str(&self) -> &str {
            &self.address
        }

        /// The part after the `@`, sliced from `address` without searching.
        pub fn domain(&self) -> &str {
            &self.address[self.at + 1..]
        }
    }
}

use account::{Email, Username, UsernameError};

#[derive(Debug)]
struct Account {
    username: Username,
    email: Email,
}

/// "Welcome, <username>! We sent a link to <email>."
fn welcome(account: &Account) -> String {
    format!(
        "Welcome, {}! We sent a link to {}.",
        account.username.as_str(),
        account.email.as_str()
    )
}

/// Whether two addresses are at the same domain (case doesn't matter).
fn same_domain(a: &Email, b: &Email) -> bool {
    a.domain() == b.domain()
}

fn main() {
    for raw in ["  Ferris_42 ", "x", "crab-man", "2fast"] {
        match Username::parse(raw) {
            Ok(name) => println!("{raw:?} -> {}", name.as_str()),
            Err(UsernameError::Length(n)) => println!("{raw:?} -> {n} characters"),
            Err(err) => println!("{raw:?} -> {err:?}"),
        }
    }
    let email = Email::parse("ferris@Rust-Lang.ORG");
    println!("{email:?}");
    if let (Ok(username), Some(email)) = (Username::parse("ferris"), email) {
        println!("{}", welcome(&Account { username, email }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames_are_normalized() {
        let name = Username::parse("  Ferris_42 ").unwrap();
        assert_eq!(name.as_str(), "ferris_42");
        assert_eq!(Username::parse("abc").unwrap().as_str(), "abc");
        assert!(Username::parse("sixteen_chars_ok").is_ok());
    }

    #[test]
    fn username_length() {
        assert_eq!(Username::parse("ab"), Err(UsernameError::Length(2)));
        assert_eq!(Username::parse("   "), Err(UsernameError::Length(0)));
        assert_eq!(
            Username::parse("seventeen_chars_x"),
            Err(UsernameError::Length(17))
        );
        assert_eq!(Username::parse("éé"), Err(UsernameError::Length(2)));
    }

    #[test]
    fn username_characters() {
        assert_eq!(
            Username::parse("crab-man"),
            Err(UsernameError::InvalidChar('-'))
        );
        assert_eq!(
            Username::parse("two words"),
            Err(UsernameError::InvalidChar(' '))
        );
        assert_eq!(
            Username::parse("-leading"),
            Err(UsernameError::InvalidChar('-'))
        );
        assert_eq!(Username::parse("2fast"), Err(UsernameError::BadStart));
        assert_eq!(Username::parse("_hidden"), Err(UsernameError::BadStart));
    }

    #[test]
    fn emails() {
        let email = Email::parse("  Ferris@Rust-Lang.ORG ").unwrap();
        assert_eq!(email.as_str(), "Ferris@rust-lang.org");
        assert_eq!(email.domain(), "rust-lang.org");
        assert!(Email::parse("a@b.c").is_some());
    }

    #[test]
    fn invalid_emails() {
        for raw in [
            "ferris",
            "@rust.org",
            "ferris@",
            "ferris@rust",
            "ferris@.rust.org",
            "ferris@rust.org.",
            "a@b@c.org",
            "",
        ] {
            assert_eq!(Email::parse(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn welcomes_and_domains() {
        let account = Account {
            username: Username::parse("Ferris").unwrap(),
            email: Email::parse("ferris@rust.example").unwrap(),
        };
        assert_eq!(
            welcome(&account),
            "Welcome, ferris! We sent a link to ferris@rust.example."
        );
        let other = Email::parse("Crab@RUST.example").unwrap();
        assert!(same_domain(&account.email, &other));
        let elsewhere = Email::parse("ferris@trust.example").unwrap();
        assert!(!same_domain(&account.email, &elsewhere));
    }
}

//! # Parse, don't validate
//!
//! Instead of checking a `String` in one place and hoping every other
//! function remembers to, parse it once into a type that can only hold
//! valid values. The types live in their own module with private fields,
//! so `Username::parse` and `Email::parse` are the only ways to get one.
//! Everything downstream takes them and checks nothing. Implement the
//! parsers and accessors, and keep the fields private.

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
            todo!("parse {raw:?} as a username")
        }

        pub fn as_str(&self) -> &str {
            todo!("borrow the name in {self:?}")
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
            todo!("parse {raw:?} as an email address")
        }

        pub fn as_str(&self) -> &str {
            todo!("borrow the address in {self:?}")
        }

        /// The part after the `@`, sliced from `address` without searching.
        pub fn domain(&self) -> &str {
            todo!("slice the domain out of {self:?}")
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
    todo!("welcome {account:?}")
}

/// Whether two addresses are at the same domain (case doesn't matter).
fn same_domain(a: &Email, b: &Email) -> bool {
    todo!("compare the domains of {a:?} and {b:?}")
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

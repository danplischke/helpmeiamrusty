//! # Binding modes: matching through references (reference solution)
//!
//! - Borrow the scrutinee (`match &contact.nickname`) and every binding
//!   becomes a reference. `match contact.nickname` names the place itself,
//!   so `Some(nick)` tried to move the `String` out from behind `&Contact`
//!   (E0507). The compiler's "consider borrowing here" is the fix.
//! - `as_deref()` turns `&Option<String>` into `Option<&str>`:
//!   `contact.nickname.as_deref().unwrap_or(&contact.name)` is the
//!   one-line `display_name`.
//! - `Some(ref nick)` was redundant before 2024 and is an error now: once
//!   a pattern borrows implicitly, `ref`, `mut` and `&` inside it are
//!   reserved (the edition guide's "match ergonomics reservations"), to
//!   leave room for better rules later.
//! - `&verified` failed for a different reason (E0308): a `&` pattern
//!   matches a reference, and the field is a plain `bool`. Write
//!   `*verified`, or take the reference apart up front:
//!   `for &(ref address, verified) in &contact.emails`.
//! - `for (address, verified) in &mut contact.emails` binds `&mut String`
//!   and `&mut bool`. The binding mode follows the reference you match on.

#[derive(Debug)]
struct Contact {
    name: String,
    nickname: Option<String>,
    /// Each address with whether it has been verified.
    emails: Vec<(String, bool)>,
}

/// The nickname if there is one, otherwise the name.
fn display_name(contact: &Contact) -> &str {
    match &contact.nickname {
        Some(nick) => nick,
        None => &contact.name,
    }
}

/// The nickname's length in characters, or 0 without one.
fn nickname_len(contact: &Contact) -> usize {
    match &contact.nickname {
        Some(nick) => nick.chars().count(),
        None => 0,
    }
}

/// The first verified address.
fn verified_email(contact: &Contact) -> Option<&str> {
    for (address, verified) in &contact.emails {
        if *verified {
            return Some(address);
        }
    }
    None
}

/// Marks every address at `domain` (the part after the `@`) as verified.
fn verify_domain(contact: &mut Contact, domain: &str) {
    for (address, verified) in &mut contact.emails {
        if address
            .rsplit_once('@')
            .is_some_and(|(_, host)| host == domain)
        {
            *verified = true;
        }
    }
}

fn ferris() -> Contact {
    Contact {
        name: "Ferris".to_string(),
        nickname: Some("crab".to_string()),
        emails: vec![
            ("ferris@old.example".to_string(), false),
            ("ferris@rust.example".to_string(), true),
            ("crab@rust.example".to_string(), true),
        ],
    }
}

fn main() {
    let mut contact = ferris();
    println!(
        "{} ({} chars)",
        display_name(&contact),
        nickname_len(&contact)
    );
    println!("{:?}", verified_email(&contact));
    verify_domain(&mut contact, "old.example");
    println!("{:?}", contact.emails);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_prefers_the_nickname() {
        let contact = ferris();
        assert_eq!(display_name(&contact), "crab");
        let formal = Contact {
            nickname: None,
            ..ferris()
        };
        assert_eq!(display_name(&formal), "Ferris");
        assert!(formal.nickname.is_none());
    }

    #[test]
    fn nickname_length_counts_chars() {
        assert_eq!(nickname_len(&ferris()), 4);
        let czech = Contact {
            nickname: Some("Krabík".to_string()),
            ..ferris()
        };
        assert_eq!(nickname_len(&czech), 6);
        let formal = Contact {
            nickname: None,
            ..ferris()
        };
        assert_eq!(nickname_len(&formal), 0);
    }

    #[test]
    fn finds_the_first_verified_email() {
        assert_eq!(verified_email(&ferris()), Some("ferris@rust.example"));
        let unverified = Contact {
            emails: vec![("a@b.example".to_string(), false)],
            ..ferris()
        };
        assert_eq!(verified_email(&unverified), None);
    }

    #[test]
    fn verifies_a_whole_domain() {
        let mut contact = ferris();
        contact
            .emails
            .push(("ferris@trust.example".to_string(), false));
        verify_domain(&mut contact, "rust.example");
        assert!(!contact.emails[3].1, "trust.example is a different domain");
        verify_domain(&mut contact, "old.example");
        assert!(contact.emails[0].1);
        assert_eq!(verified_email(&contact), Some("ferris@old.example"));
    }
}

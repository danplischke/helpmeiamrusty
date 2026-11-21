//! # Moves, borrows and copies
//!
//! These functions take ownership of things they only need to read, so the
//! callers lose their values. Change the signatures (and bodies) to borrow
//! instead. The tests show how the functions must be callable.
//!
//! Rule: no `.clone()`. Cloning would compile, but it hides the real fix.

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    logins: u32,
}

/// Total number of bytes across all words.
fn total_len(words: Vec<String>) -> usize {
    words.iter().map(String::len).sum()
}

/// "HELLO" from "hello".
fn shout(s: String) -> String {
    s.to_uppercase()
}

fn greeting(user: &User) -> String {
    let name = user.name;
    format!("Hello, {name}! You've logged in {} times.", user.logins)
}

/// Pushes "<name> <<email>>" for every user who has logged in at least
/// once, and returns how many users there were in total.
fn active_labels(users: &Vec<User>, labels: &mut Vec<String>) -> usize {
    for user in *users {
        if user.logins > 0 {
            labels.push(format!("{} <{}>", user.name, user.email));
        }
    }
    users.len()
}

fn main() {
    let users = vec![
        User {
            name: "Ferris".to_string(),
            email: "ferris@example.com".to_string(),
            logins: 3,
        },
        User {
            name: "Corro".to_string(),
            email: "corro@example.com".to_string(),
            logins: 0,
        },
    ];
    let mut labels = Vec::new();
    let total = active_labels(&users, &mut labels);
    println!("{total} users, active: {labels:?}");
    println!("{}", greeting(&users[0]));
    println!("{}", shout(&users[1].name));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_users() -> Vec<User> {
        vec![
            User {
                name: "Ferris".to_string(),
                email: "ferris@example.com".to_string(),
                logins: 3,
            },
            User {
                name: "Corro".to_string(),
                email: "corro@example.com".to_string(),
                logins: 0,
            },
        ]
    }

    #[test]
    fn total_len_borrows() {
        let words = vec![String::from("borrow"), String::from("checker")];
        assert_eq!(total_len(&words), 13);
        assert_eq!(words.len(), 2, "the caller still owns `words`");
        assert_eq!(total_len(&words[..1]), 6, "any slice works");
    }

    #[test]
    fn shout_borrows() {
        let quiet = String::from("hello");
        assert_eq!(shout(&quiet), "HELLO");
        assert_eq!(shout("literals work too"), "LITERALS WORK TOO");
        assert_eq!(quiet, "hello");
    }

    #[test]
    fn greeting_reads_fields() {
        let users = sample_users();
        assert_eq!(
            greeting(&users[0]),
            "Hello, Ferris! You've logged in 3 times."
        );
        assert_eq!(users[0].name, "Ferris");
    }

    #[test]
    fn labels_without_consuming() {
        let users = sample_users();
        let mut labels = Vec::new();
        assert_eq!(active_labels(&users, &mut labels), 2);
        assert_eq!(labels, ["Ferris <ferris@example.com>"]);
        assert_eq!(users[1].name, "Corro", "the caller still owns `users`");
    }
}

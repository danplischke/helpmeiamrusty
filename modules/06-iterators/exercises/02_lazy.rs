//! # Lazy iterators hold on to things
//!
//! An iterator is a recipe: nothing runs until something consumes it, and
//! until then it keeps borrowing (or owning) what it reads from, closure
//! captures included. Three of these functions don't compile because of
//! that. The fourth compiles but does nothing; once the others build, read
//! the compiler's warning about it. Fix each one so it does what its doc
//! comment says.
//!
//! Rule: no `.clone()`.

#[derive(Debug, PartialEq)]
struct Message {
    to: String,
    body: String,
    sent: bool,
}

/// The average body length in bytes; `None` when there are no messages.
fn average_len(outbox: &[Message]) -> Option<f64> {
    let lens = outbox.iter().map(|m| m.body.len());
    let total: usize = lens.sum();
    let count = lens.count();
    if count == 0 {
        return None;
    }
    Some(total as f64 / count as f64)
}

/// How many messages are urgent (the body starts with '!'), and their
/// bodies, in order.
fn urgent(outbox: &[Message]) -> (usize, Vec<&str>) {
    let mut count = 0;
    let bodies = outbox.iter().filter(|m| m.body.starts_with('!')).map(|m| {
        count += 1;
        m.body.as_str()
    });
    (count, bodies.collect())
}

/// Consumes the outbox and yields its recipients, in order.
fn into_recipients(outbox: Vec<Message>) -> impl Iterator<Item = String> {
    outbox.iter().map(|m| m.to.clone())
}

/// Marks every message as sent.
fn mark_all_sent(outbox: &mut [Message]) {
    outbox.iter_mut().map(|m| m.sent = true);
}

fn message(to: &str, body: &str) -> Message {
    Message {
        to: to.to_string(),
        body: body.to_string(),
        sent: false,
    }
}

fn main() {
    let mut outbox = vec![
        message("ops", "!disk full"),
        message("ann", "lunch?"),
        message("ops", "!backup failed"),
    ];
    println!("average length: {:?}", average_len(&outbox));
    println!("urgent: {:?}", urgent(&outbox));
    mark_all_sent(&mut outbox);
    println!("all sent: {}", outbox.iter().all(|m| m.sent));
    let recipients: Vec<String> = into_recipients(outbox).collect();
    println!("recipients: {recipients:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outbox() -> Vec<Message> {
        vec![
            message("ops", "!server down"),
            message("ann", "lunch?"),
            message("cy", "!disk is full"),
            message("bob", "ok"),
        ]
    }

    #[test]
    fn averages_body_lengths() {
        assert_eq!(average_len(&outbox()), Some(8.25));
        assert_eq!(average_len(&[]), None);
    }

    #[test]
    fn counts_and_lists_urgent_messages() {
        let outbox = outbox();
        assert_eq!(urgent(&outbox), (2, vec!["!server down", "!disk is full"]));
        assert_eq!(urgent(&outbox[1..2]), (0, vec![]));
    }

    #[test]
    fn yields_recipients_in_order() {
        let recipients: Vec<String> = into_recipients(outbox()).collect();
        assert_eq!(recipients, ["ops", "ann", "cy", "bob"]);
    }

    #[test]
    fn marks_everything_sent() {
        let mut outbox = outbox();
        mark_all_sent(&mut outbox);
        assert!(outbox.iter().all(|m| m.sent));
    }
}

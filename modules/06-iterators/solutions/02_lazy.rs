//! # Lazy iterators hold on to things (reference solution)
//!
//! - Consumers (`sum`, `count`, `collect`, `for`) take the iterator by
//!   value: an iterator is single-pass. The slice already knows its length,
//!   so there's nothing to count twice. If you need two answers from one
//!   pass, `fold` with a tuple accumulator.
//! - The closure in `urgent` held `&mut count` for as long as the iterator
//!   lived (E0503). Had it compiled, `count` would have been read before the
//!   closure ever ran, giving 0. Side effects inside `map` are a smell:
//!   collect, then measure what you collected.
//! - `outbox.iter()` borrows a Vec that is dropped when the function
//!   returns, while the returned iterator still needs it (E0597, "`outbox`
//!   does not live long enough"). `into_iter()` moves the Vec into the
//!   iterator, which then yields owned items: no clone needed.
//! - An unconsumed `map` is a no-op, and rustc warns: "iterators are lazy
//!   and do nothing unless consumed". For side effects, use a `for` loop.

#[derive(Debug, PartialEq)]
struct Message {
    to: String,
    body: String,
    sent: bool,
}

/// The average body length in bytes; `None` when there are no messages.
fn average_len(outbox: &[Message]) -> Option<f64> {
    if outbox.is_empty() {
        return None;
    }
    let total: usize = outbox.iter().map(|m| m.body.len()).sum();
    Some(total as f64 / outbox.len() as f64)
}

/// How many messages are urgent (the body starts with '!'), and their
/// bodies, in order.
fn urgent(outbox: &[Message]) -> (usize, Vec<&str>) {
    let bodies: Vec<&str> = outbox
        .iter()
        .filter(|m| m.body.starts_with('!'))
        .map(|m| m.body.as_str())
        .collect();
    (bodies.len(), bodies)
}

/// Consumes the outbox and yields its recipients, in order.
fn into_recipients(outbox: Vec<Message>) -> impl Iterator<Item = String> {
    outbox.into_iter().map(|m| m.to)
}

/// Marks every message as sent.
fn mark_all_sent(outbox: &mut [Message]) {
    for m in outbox {
        m.sent = true;
    }
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

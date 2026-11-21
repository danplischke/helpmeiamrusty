//! # Default methods, supertraits and associated consts (reference solution)
//!
//! - Inside a default method, `Self` is "some type that implements
//!   `Channel`", so all you can use is what the trait and its supertraits
//!   promise: `format!("[{self}] ..")` works because `Channel: Display`.
//! - `Self::MAX_CHARS` reads the implementing type's value: the default
//!   unless the impl overrides it, as `Email` does. From outside it's
//!   `Email::MAX_CHARS`, or `C::MAX_CHARS` for a generic `C: Channel`.
//! - A default method that calls another trait method gets the overriding
//!   version, so `Pager` only had to override `broadcast`.
//! - `-> impl Iterator<Item = &str>` in a trait (Rust 1.75) lets each impl
//!   return its own iterator type without boxing. The price: the trait is no
//!   longer dyn compatible. So is any trait with an associated const, so
//!   `Box<dyn Channel>` wouldn't compile either way.
//! - `.chars().count()`, not `.len()`: the limit is in characters.

use std::fmt;

/// Somewhere to send short text messages. Every channel can be displayed,
/// and the default methods rely on that.
trait Channel: fmt::Display {
    /// The longest line, in characters, that this channel accepts.
    const MAX_CHARS: usize = 160;

    /// Hands a finished line to the channel.
    fn deliver(&mut self, line: String);

    /// Every line this channel still holds, oldest first. Each impl returns
    /// its own iterator type behind `impl Iterator`.
    fn history(&self) -> impl Iterator<Item = &str>;

    /// Formats `text` as "[<channel>] <text>", where <channel> is the
    /// channel's `Display` form, and delivers that line. If the line is
    /// longer than `MAX_CHARS` characters, delivers nothing and returns
    /// false.
    fn send(&mut self, text: &str) -> bool {
        let line = format!("[{self}] {text}");
        if line.chars().count() > Self::MAX_CHARS {
            return false;
        }
        self.deliver(line);
        true
    }

    /// Sends each text in turn and returns how many were delivered.
    fn broadcast(&mut self, texts: &[&str]) -> usize {
        texts.iter().filter(|text| self.send(text)).count()
    }

    /// The most recent line in `history`, if there is one.
    fn last_line(&self) -> Option<&str> {
        self.history().last()
    }
}

/// Text messages. Uses the default limit.
struct Sms {
    number: String,
    outbox: Vec<String>,
}

/// Email accepts lines of up to 1000 characters.
struct Email {
    address: String,
    outbox: Vec<String>,
}

/// A pager holds only the latest line and beeps once per delivery.
/// Broadcasting to a pager would only leave the last text on screen anyway,
/// so its `broadcast` sends only the last text: one beep, and it returns 1
/// if that text was delivered, 0 if not (or if there were no texts).
struct Pager {
    screen: Option<String>,
    beeps: u32,
}

impl fmt::Display for Sms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sms:{}", self.number)
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mail:{}", self.address)
    }
}

impl fmt::Display for Pager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("pager")
    }
}

impl Channel for Sms {
    fn deliver(&mut self, line: String) {
        self.outbox.push(line);
    }

    fn history(&self) -> impl Iterator<Item = &str> {
        self.outbox.iter().map(String::as_str)
    }
}

impl Channel for Email {
    const MAX_CHARS: usize = 1000;

    fn deliver(&mut self, line: String) {
        self.outbox.push(line);
    }

    fn history(&self) -> impl Iterator<Item = &str> {
        self.outbox.iter().map(String::as_str)
    }
}

impl Channel for Pager {
    fn deliver(&mut self, line: String) {
        self.screen = Some(line);
        self.beeps += 1;
    }

    fn history(&self) -> impl Iterator<Item = &str> {
        self.screen.as_deref().into_iter()
    }

    fn broadcast(&mut self, texts: &[&str]) -> usize {
        texts.last().map_or(0, |text| usize::from(self.send(text)))
    }
}

fn main() {
    let mut sms = Sms {
        number: "+15550100".to_string(),
        outbox: Vec::new(),
    };
    let mut pager = Pager {
        screen: None,
        beeps: 0,
    };
    println!("{}", sms.broadcast(&["deploy started", "deploy done"]));
    println!("{}", pager.broadcast(&["disk at 90%", "disk at 95%"]));
    println!("{:?} / {:?}", sms.last_line(), pager.last_line());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sms() -> Sms {
        Sms {
            number: "+15550100".to_string(),
            outbox: Vec::new(),
        }
    }

    fn email() -> Email {
        Email {
            address: "ops@example.com".to_string(),
            outbox: Vec::new(),
        }
    }

    fn pager() -> Pager {
        Pager {
            screen: None,
            beeps: 0,
        }
    }

    #[test]
    fn send_prefixes_the_channel() {
        let mut sms = sms();
        assert!(sms.send("hello"));
        assert_eq!(sms.history().collect::<Vec<_>>(), ["[sms:+15550100] hello"]);
        let mut email = email();
        assert!(email.send("disk full"));
        assert_eq!(email.last_line(), Some("[mail:ops@example.com] disk full"));
    }

    #[test]
    fn lines_over_the_limit_are_refused() {
        let mut sms = sms();
        let prefix = "[sms:+15550100] ".len();
        let fits = "x".repeat(160 - prefix);
        assert!(sms.send(&fits));
        assert!(!sms.send(&format!("{fits}x")));
        assert_eq!(sms.history().count(), 1);
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        let mut sms = sms();
        let prefix = "[sms:+15550100] ".len();
        assert!(sms.send(&"é".repeat(160 - prefix)));
    }

    #[test]
    fn email_has_its_own_limit() {
        assert_eq!(Sms::MAX_CHARS, 160);
        assert_eq!(Email::MAX_CHARS, 1000);
        let mut email = email();
        assert!(email.send(&"y".repeat(900)));
        assert!(!email.send(&"y".repeat(1000)));
    }

    #[test]
    fn broadcast_counts_deliveries() {
        let mut sms = sms();
        let long = "z".repeat(200);
        assert_eq!(sms.broadcast(&["a", &long, "b"]), 2);
        assert_eq!(sms.last_line(), Some("[sms:+15550100] b"));
        assert_eq!(sms.broadcast(&[]), 0);
        assert_eq!(sms.history().count(), 2);
    }

    #[test]
    fn a_pager_shows_the_latest_line() {
        let mut pager = pager();
        assert_eq!(pager.last_line(), None);
        assert!(pager.send("one"));
        assert!(pager.send("two"));
        assert_eq!(pager.history().collect::<Vec<_>>(), ["[pager] two"]);
        assert_eq!(pager.beeps, 2);
    }

    #[test]
    fn a_pager_broadcasts_only_the_last_text() {
        let mut pager = pager();
        assert_eq!(pager.broadcast(&["one", "two", "three"]), 1);
        assert_eq!(pager.last_line(), Some("[pager] three"));
        assert_eq!(pager.beeps, 1);
        let long = "z".repeat(200);
        assert_eq!(pager.broadcast(&["short", &long]), 0);
        assert_eq!(pager.broadcast(&[]), 0);
        assert_eq!(pager.beeps, 1);
    }
}

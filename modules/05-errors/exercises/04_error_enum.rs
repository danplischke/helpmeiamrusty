//! # A hand-written error type
//!
//! A library's error is usually an enum with one variant per thing a
//! caller might handle differently. To be a real error it implements
//! `Display` (a short, lowercase message) and `std::error::Error`, whose
//! `source()` returns the underlying cause. The convention: put the cause
//! in `source()` *or* in your message, never both, or a report that walks
//! the chain prints it twice. Implement the impls, then `load_quotas`.

use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::num::ParseIntError;

#[derive(Debug)]
enum QuotaError {
    /// "could not read the quota file", caused by the `io::Error`.
    Io(io::Error),
    /// "line <line>: expected \"<user> <limit>\"". No cause.
    Malformed { line: usize },
    /// "line <line>: invalid limit", caused by the `ParseIntError`.
    BadLimit { line: usize, source: ParseIntError },
    /// "line <line>: user <user> is listed twice". No cause.
    Duplicate { line: usize, user: String },
}

impl fmt::Display for QuotaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f; // only here to silence "unused variable"
        todo!("describe {self:?}")
    }
}

impl Error for QuotaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!("the cause of {self:?}")
    }
}

/// Lets `?` turn an `io::Error` into `QuotaError::Io`.
impl From<io::Error> for QuotaError {
    fn from(e: io::Error) -> Self {
        todo!("wrap {e:?}")
    }
}

#[derive(Debug, PartialEq)]
struct Quota {
    user: String,
    limit: u32,
}

/// Reads quotas, one per line: `<user> <limit>`, exactly two
/// whitespace-separated fields. Blank lines are skipped. Lines are numbered
/// from 1, blank ones included. Returns the quotas in file order, or the
/// first error.
fn load_quotas(reader: impl Read) -> Result<Vec<Quota>, QuotaError> {
    let _ = reader; // only here to silence "unused variable"
    todo!("read the quotas")
}

/// Every message in an error's chain, outermost first.
fn chain(error: &dyn Error) -> Vec<String> {
    let mut messages = vec![error.to_string()];
    let mut current = error.source();
    while let Some(cause) = current {
        messages.push(cause.to_string());
        current = cause.source();
    }
    messages
}

fn main() {
    let text = "alice 10\n\nbob 25\n";
    println!("{:?}", load_quotas(text.as_bytes()));
    for bad in ["alice 10\nbob lots", "alice 10\nalice 20", "carol"] {
        if let Err(e) = load_quotas(bad.as_bytes()) {
            println!("{}", chain(&e).join(": "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reader whose disk has gone away.
    struct Unplugged;

    impl Read for Unplugged {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("disk unplugged"))
        }
    }

    fn quota(user: &str, limit: u32) -> Quota {
        Quota {
            user: user.to_string(),
            limit,
        }
    }

    #[test]
    fn loads_quotas() {
        let text = "alice 10\n\n  bob   25  \n";
        let quotas = load_quotas(text.as_bytes()).unwrap();
        assert_eq!(quotas, [quota("alice", 10), quota("bob", 25)]);
        assert!(load_quotas("".as_bytes()).unwrap().is_empty());
    }

    #[test]
    fn malformed_lines() {
        for text in ["alice", "alice 10\n\nbob 1 2"] {
            let err = load_quotas(text.as_bytes()).unwrap_err();
            let line = text.lines().count();
            assert_eq!(
                chain(&err),
                [format!("line {line}: expected \"<user> <limit>\"")]
            );
        }
    }

    #[test]
    fn bad_limits_keep_their_cause() {
        let err = load_quotas("alice 10\nbob -5".as_bytes()).unwrap_err();
        assert!(matches!(err, QuotaError::BadLimit { line: 2, .. }));
        assert_eq!(
            chain(&err),
            ["line 2: invalid limit", "invalid digit found in string"]
        );
        let cause = err.source().unwrap();
        assert!(cause.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn duplicates() {
        let err = load_quotas("alice 10\nbob 3\nalice 20".as_bytes()).unwrap_err();
        assert_eq!(chain(&err), ["line 3: user alice is listed twice"]);
    }

    #[test]
    fn io_errors_convert_and_keep_their_cause() {
        let err = load_quotas(Unplugged).unwrap_err();
        assert!(matches!(err, QuotaError::Io(_)));
        assert_eq!(
            chain(&err),
            ["could not read the quota file", "disk unplugged"]
        );
        let err = QuotaError::from(io::Error::from(io::ErrorKind::NotFound));
        let cause = err.source().unwrap().downcast_ref::<io::Error>().unwrap();
        assert_eq!(cause.kind(), io::ErrorKind::NotFound);
    }
}

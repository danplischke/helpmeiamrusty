//! # Conversion traits (reference solution)
//!
//! - Implement `From`/`TryFrom`, never `Into`/`TryInto`: the blanket
//!   `impl<T, U: From<T>> Into<U> for T` only works in that direction, and
//!   clippy flags `impl Into` (`from_over_into`).
//! - `u16::try_from(n)` does the range check for you. `n as u16` would
//!   silently wrap 65536 to 0. Matching `Ok(0)` before `Ok(port)` handles
//!   the one value `u16` allows and `Port` doesn't.
//! - `impl Into<String>` lets callers pass `"db"` or a `String` they no
//!   longer need, which is moved in without a copy. `impl AsRef<str>`, or
//!   `&[S]` with `S: AsRef<str>`, is for strings you only read.
//! - `FromStr` with `?` and `ok_or`/`map_err` keeps parsing linear: each
//!   step either produces the next value or returns its own error.
//!   `map_err(EndpointError::Port)` uses the tuple variant as a function.

use std::str::FromStr;

/// A TCP port. Port 0 isn't allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Port(u16);

#[derive(Debug, PartialEq, Eq)]
enum PortError {
    /// The number was 0.
    Zero,
    /// The number was negative or above 65535.
    OutOfRange(i64),
}

/// A checked conversion from whatever integer the caller has.
impl TryFrom<i64> for Port {
    type Error = PortError;

    fn try_from(n: i64) -> Result<Self, Self::Error> {
        match u16::try_from(n) {
            Ok(0) => Err(PortError::Zero),
            Ok(port) => Ok(Port(port)),
            Err(_) => Err(PortError::OutOfRange(n)),
        }
    }
}

/// Every port is a valid `u16`, so this direction can't fail.
impl From<Port> for u16 {
    fn from(port: Port) -> u16 {
        port.0
    }
}

/// Where to connect.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    host: String,
    port: Port,
    tls: bool,
}

/// "localhost", port 8080, no TLS.
impl Default for Endpoint {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: Port(8080),
            tls: false,
        }
    }
}

impl Endpoint {
    /// An endpoint for `host`, with everything else from `Default`. Given a
    /// `String`, it keeps that `String` rather than copying it.
    fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EndpointError {
    /// No ':' between host and port.
    MissingPort,
    /// Nothing before the ':'.
    EmptyHost,
    /// The port isn't a whole number.
    NotANumber,
    /// The port is a number, but not a valid `Port`.
    Port(PortError),
}

/// Parses "host:port", optionally prefixed with "https://", which sets
/// `tls`. The port comes after the *last* ':'. Errors are checked in the
/// order the variants of `EndpointError` are listed.
impl FromStr for Endpoint {
    type Err = EndpointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tls, rest) = match s.strip_prefix("https://") {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        let (host, port) = rest.rsplit_once(':').ok_or(EndpointError::MissingPort)?;
        if host.is_empty() {
            return Err(EndpointError::EmptyHost);
        }
        let number: i64 = port.parse().map_err(|_| EndpointError::NotANumber)?;
        let port = Port::try_from(number).map_err(EndpointError::Port)?;
        Ok(Self {
            host: host.to_string(),
            port,
            tls,
        })
    }
}

/// True for "localhost", "127.0.0.1" and "::1".
fn is_loopback(host: impl AsRef<str>) -> bool {
    matches!(host.as_ref(), "localhost" | "127.0.0.1" | "::1")
}

/// How many of `hosts` are loopback addresses. Works on `&[&str]` and
/// `&[String]` alike.
fn count_loopback<S: AsRef<str>>(hosts: &[S]) -> usize {
    hosts.iter().filter(|host| is_loopback(host)).count()
}

fn main() {
    println!("{:?}", Port::try_from(70_000));
    println!("{:?}", Endpoint::new("db.internal"));
    println!("{:?}", "https://api.example.com:443".parse::<Endpoint>());
    println!("{}", count_loopback(&["localhost", "example.com"]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_are_checked() {
        assert_eq!(Port::try_from(443), Ok(Port(443)));
        assert_eq!(Port::try_from(65535), Ok(Port(65535)));
        assert_eq!(Port::try_from(0), Err(PortError::Zero));
        assert_eq!(Port::try_from(65536), Err(PortError::OutOfRange(65536)));
        assert_eq!(Port::try_from(-1), Err(PortError::OutOfRange(-1)));
        let port: Result<Port, _> = 8080_i64.try_into();
        assert_eq!(port, Ok(Port(8080)));
    }

    #[test]
    fn ports_convert_back_to_u16() {
        assert_eq!(u16::from(Port(22)), 22);
        let n: u16 = Port(5432).into();
        assert_eq!(n, 5432);
    }

    #[test]
    fn default_endpoint() {
        let endpoint = Endpoint::default();
        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, Port(8080));
        assert!(!endpoint.tls);
    }

    #[test]
    fn new_takes_str_or_string() {
        let endpoint = Endpoint::new("db");
        assert_eq!(endpoint.host, "db");
        assert_eq!(endpoint.port, Port(8080));
        let host = String::from("cache.internal");
        let buffer = host.as_ptr();
        let endpoint = Endpoint::new(host);
        assert_eq!(endpoint.host.as_ptr(), buffer, "moved in, not copied");
    }

    #[test]
    fn parses_endpoints() {
        let db = Endpoint {
            host: "db.internal".to_string(),
            port: Port(5432),
            tls: false,
        };
        assert_eq!("db.internal:5432".parse(), Ok(db));
        let api: Endpoint = "https://api.example.com:443".parse().unwrap();
        assert_eq!(api.host, "api.example.com");
        assert_eq!(api.port, Port(443));
        assert!(api.tls);
    }

    #[test]
    fn rejects_bad_endpoints() {
        let parse = |s: &str| s.parse::<Endpoint>();
        assert_eq!(parse("localhost"), Err(EndpointError::MissingPort));
        assert_eq!(parse(":8080"), Err(EndpointError::EmptyHost));
        assert_eq!(parse("https://:443"), Err(EndpointError::EmptyHost));
        assert_eq!(parse("db:http"), Err(EndpointError::NotANumber));
        assert_eq!(parse("db:"), Err(EndpointError::NotANumber));
        assert_eq!(parse("db:0"), Err(EndpointError::Port(PortError::Zero)));
        assert_eq!(
            parse("db:70000"),
            Err(EndpointError::Port(PortError::OutOfRange(70000)))
        );
    }

    #[test]
    fn loopback_checks_take_any_string() {
        assert!(is_loopback("localhost"));
        assert!(is_loopback(String::from("::1")));
        let endpoint = Endpoint::new("127.0.0.1");
        assert!(is_loopback(&endpoint.host));
        assert!(!is_loopback("example.com"));
        assert_eq!(
            count_loopback(&["127.0.0.1", "example.com", "localhost"]),
            2
        );
        assert_eq!(count_loopback(&[String::from("db")]), 0);
        assert_eq!(count_loopback::<&str>(&[]), 0);
    }
}

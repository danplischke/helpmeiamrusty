//! # What `?` does with an error
//!
//! On `Err(e)`, `expr?` returns `Err(From::from(e))`: it converts `e` into
//! the function's error type, so that type needs a `From` impl for every
//! error you apply `?` to. And `?` only works on the kind of value the
//! function returns: an `Option` in an `Option` function, a `Result` in a
//! `Result` function. Make this compile without changing what it does.

use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
enum EndpointError {
    MissingPort,
    EmptyHost,
    BadPort(ParseIntError),
    PrivilegedPort(u16),
}

#[derive(Debug, PartialEq)]
struct Endpoint {
    host: String,
    port: u16,
}

/// Parses "host:port", splitting at the last ':'. Errors: `MissingPort`
/// without a ':', `EmptyHost` if nothing comes before it, `BadPort` if the
/// port isn't a `u16`, and `PrivilegedPort` for ports below 1024.
fn parse_endpoint(s: &str) -> Result<Endpoint, EndpointError> {
    let (host, port) = s.rsplit_once(':')?;
    if host.is_empty() {
        return Err(EndpointError::EmptyHost);
    }
    let port = port.parse::<u16>()?;
    if port < 1024 {
        return Err(EndpointError::PrivilegedPort(port));
    }
    Ok(Endpoint {
        host: host.to_string(),
        port,
    })
}

/// Just the port of "host:port", for callers that don't care why it's
/// missing: `None` without a ':' or if the port isn't a `u16`. Any port
/// number counts here, privileged or not.
fn port_of(s: &str) -> Option<u16> {
    let (_, port) = s.rsplit_once(':')?;
    let port = port.parse()?;
    Some(port)
}

fn main() {
    println!("{:?}", parse_endpoint("db.internal:5432"));
    println!("{:?}", parse_endpoint("db.internal"));
    println!("{:?}", parse_endpoint("db.internal:http"));
    println!("{:?}", port_of("localhost:80"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_endpoints() {
        let endpoint = parse_endpoint("db.internal:5432").unwrap();
        assert_eq!(endpoint.host, "db.internal");
        assert_eq!(endpoint.port, 5432);
        assert_eq!(parse_endpoint("[::1]:8080").unwrap().host, "[::1]");
    }

    #[test]
    fn bad_endpoints() {
        assert_eq!(
            parse_endpoint("db.internal"),
            Err(EndpointError::MissingPort)
        );
        assert_eq!(parse_endpoint(":5432"), Err(EndpointError::EmptyHost));
        assert_eq!(
            parse_endpoint("db:22"),
            Err(EndpointError::PrivilegedPort(22))
        );
        assert!(matches!(
            parse_endpoint("db:http"),
            Err(EndpointError::BadPort(_))
        ));
        assert!(matches!(
            parse_endpoint("db:70000"),
            Err(EndpointError::BadPort(_))
        ));
    }

    #[test]
    fn parse_errors_convert_with_from() {
        let parse_error = "http".parse::<u16>().unwrap_err();
        let converted = EndpointError::from(parse_error.clone());
        assert_eq!(converted, EndpointError::BadPort(parse_error));
    }

    #[test]
    fn ports() {
        assert_eq!(port_of("localhost:80"), Some(80));
        assert_eq!(port_of("localhost"), None);
        assert_eq!(port_of("localhost:eighty"), None);
    }
}

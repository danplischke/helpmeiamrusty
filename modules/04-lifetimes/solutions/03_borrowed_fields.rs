//! # Borrow instead of copying (reference solution)
//!
//! - `Request<'a>` reads "a request that borrows from something living for
//!   `'a`". Every field is a slice of the raw text; the only allocation
//!   left is the headers `Vec`.
//! - `parse_request(raw: &str) -> Option<Request<'_>>`: one input lifetime,
//!   so elision ties the request to `raw`. The `'_` keeps that visible.
//! - Inside `impl<'a> Request<'a>`, methods can return `&'a str`: the
//!   lifetime of the *data*, not of `&self`. That is what lets a header
//!   value outlive the `Request` it came from (exercise 04 drills it).
//!   `.map(|&(_, value)| value)` copies the `&'a str` out of the tuple.
//! - The price: a borrowing struct can't outlive its input, so you can't
//!   keep it after the buffer is reused or send it to a `'static` thread.
//!   When you need that, own the data (`String`), or own one buffer and
//!   store ranges into it.

#[derive(Debug, PartialEq)]
struct Request<'a> {
    method: &'a str,
    path: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: &'a str,
}

/// Parses a request: a request line "<METHOD> <PATH> <VERSION>", then
/// "Name: value" header lines, then an empty line and the body (lines end
/// with '\n'). Without an empty line, there's no body. Header names and
/// values are trimmed. `None` if the request line doesn't have exactly
/// three parts or a header line has no colon.
fn parse_request(raw: &str) -> Option<Request<'_>> {
    let (head, body) = raw.split_once("\n\n").unwrap_or((raw, ""));
    let mut lines = head.lines();
    let mut parts = lines.next()?.split_whitespace();
    let (method, path, _version) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let mut headers = Vec::new();
    for line in lines {
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim(), value.trim()));
    }
    Some(Request {
        method,
        path,
        headers,
        body,
    })
}

impl<'a> Request<'a> {
    /// The value of the first header called `name`, ignoring ASCII case.
    fn header(&self, name: &str) -> Option<&'a str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|&(_, value)| value)
    }

    /// The path without its query string: "/search?q=x" gives "/search".
    fn route(&self) -> &'a str {
        self.path
            .split_once('?')
            .map_or(self.path, |(route, _)| route)
    }
}

fn main() {
    let raw = "GET /search?q=lifetimes HTTP/1.1\nHost: example.com\n\n";
    if let Some(request) = parse_request(raw) {
        println!(
            "{} {} host={:?}",
            request.method,
            request.route(),
            request.header("host")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str =
        "GET /search?q=borrow HTTP/1.1\nHost: example.com\nAccept:  text/html \n\nhello";

    /// Whether `inner` lies inside `outer`'s bytes.
    fn points_into(outer: &str, inner: &str) -> bool {
        let start = outer.as_ptr() as usize;
        let at = inner.as_ptr() as usize;
        start <= at && at + inner.len() <= start + outer.len()
    }

    #[test]
    fn parses_a_request() {
        let request = parse_request(RAW).unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/search?q=borrow");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.header("accept"), Some("text/html"));
        assert_eq!(request.header("HOST"), Some("example.com"));
        assert_eq!(request.header("cookie"), None);
        assert_eq!(request.route(), "/search");
        assert_eq!(request.body, "hello");
    }

    #[test]
    fn no_headers_no_body() {
        let request = parse_request("HEAD / HTTP/1.1").unwrap();
        assert!(request.headers.is_empty());
        assert_eq!(request.body, "");
        assert_eq!(request.route(), "/");
    }

    #[test]
    fn rejects_malformed_requests() {
        assert_eq!(parse_request(""), None);
        assert_eq!(parse_request("GET /"), None);
        assert_eq!(parse_request("GET / HTTP/1.1 extra"), None);
        assert_eq!(parse_request("GET / HTTP/1.1\nno colon here"), None);
    }

    #[test]
    fn values_outlive_the_request() {
        let raw = String::from(RAW);
        let (host, route) = {
            let request = parse_request(&raw).unwrap();
            (request.header("host"), request.route())
        };
        assert_eq!(host, Some("example.com"));
        assert_eq!(route, "/search");
    }

    #[test]
    fn fields_point_into_the_input() {
        let raw = String::from(RAW);
        let request = parse_request(&raw).unwrap();
        assert!(points_into(&raw, &request.method), "method was copied");
        assert!(points_into(&raw, &request.path), "path was copied");
        assert!(points_into(&raw, &request.body), "body was copied");
        for (name, value) in &request.headers {
            assert!(points_into(&raw, name), "header name was copied");
            assert!(points_into(&raw, value), "header value was copied");
        }
    }
}

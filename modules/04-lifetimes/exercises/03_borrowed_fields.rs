//! # Borrow instead of copying
//!
//! `parse_request` copies every piece of a request into a new `String`,
//! although a parsed request never outlives the raw text in this program.
//! Make `Request` borrow from the raw text instead: give it a lifetime
//! parameter and `&'a str` fields. The tests check that values outlive the
//! `Request` and that the fields point into the input, so copies won't pass.

#[derive(Debug, PartialEq)]
struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

/// Parses a request: a request line "<METHOD> <PATH> <VERSION>", then
/// "Name: value" header lines, then an empty line and the body (lines end
/// with '\n'). Without an empty line, there's no body. Header names and
/// values are trimmed. `None` if the request line doesn't have exactly
/// three parts or a header line has no colon.
fn parse_request(raw: &str) -> Option<Request> {
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
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    Some(Request {
        method: method.to_string(),
        path: path.to_string(),
        headers,
        body: body.to_string(),
    })
}

impl Request {
    /// The value of the first header called `name`, ignoring ASCII case.
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The path without its query string: "/search?q=x" gives "/search".
    fn route(&self) -> &str {
        self.path
            .split_once('?')
            .map_or(self.path.as_str(), |(route, _)| route)
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

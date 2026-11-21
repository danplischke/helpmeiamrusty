//! # Flat control flow (reference solution)
//!
//! - In a function returning `Option`, "bind or return `None`" is `?`.
//!   `let ... else` is for every other way out: `continue` here, or
//!   returning a default or a specific error.
//! - `cert.zip(key)` is `Some((cert, key))` only if both are `Some`. The
//!   same thing as a pattern: `let (Some(cert), Some(key)) = (cert, key)
//!   else { return None };`. `is_some()` followed by `unwrap()` checks
//!   twice and still panics if the two drift apart.
//! - Let chains (Rust 1.88, edition 2024) turn `if let` pyramids into one
//!   condition, and the bindings stay in scope for the rest of the chain.
//!   The combinator version of `effective_port`:
//!   `env_port.and_then(|raw| raw.parse().ok()).filter(|&p| p >= 1024).unwrap_or(config.port)`.
//! - Pick whichever reads better. Combinators suit short pipelines;
//!   `let ... else` and let chains keep longer logic flat and leave room
//!   for side effects, like pushing to `bad_lines` before `continue`.

#[derive(Debug, PartialEq)]
struct Tls {
    cert: String,
    key: String,
}

#[derive(Debug, PartialEq)]
struct Config {
    host: String,
    port: u16,
    tls: Option<Tls>,
    /// 1-based numbers of the lines that couldn't be used.
    bad_lines: Vec<usize>,
}

/// Splits "key = value" into its trimmed halves at the first `=`. `None`
/// if there's no `=` or the key is empty (an empty value is fine).
fn split_pair(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value.trim()))
}

/// The TLS settings, but only if both a certificate and a key are given.
fn tls(cert: Option<&str>, key: Option<&str>) -> Option<Tls> {
    cert.zip(key).map(|(cert, key)| Tls {
        cert: cert.to_string(),
        key: key.to_string(),
    })
}

/// Reads a config. Blank lines and `#` comments are skipped. The keys are
/// `host`, `port`, `tls_cert` and `tls_key`, and a later line overrides an
/// earlier one. A line without a key and `=`, an unknown key, or a port
/// that isn't a `u16` goes into `bad_lines`. The defaults are host
/// "localhost", port 8080 and no TLS.
fn load(text: &str) -> Config {
    let mut config = Config {
        host: "localhost".to_string(),
        port: 8080,
        tls: None,
        bad_lines: Vec::new(),
    };
    let mut cert = None;
    let mut key = None;
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = split_pair(line) else {
            config.bad_lines.push(index + 1);
            continue;
        };
        match name {
            "host" => config.host = value.to_string(),
            "port" => match value.parse() {
                Ok(port) => config.port = port,
                Err(_) => config.bad_lines.push(index + 1),
            },
            "tls_cert" => cert = Some(value),
            "tls_key" => key = Some(value),
            _ => config.bad_lines.push(index + 1),
        }
    }
    config.tls = tls(cert, key);
    config
}

/// The port to listen on. `env_port` (from a `PORT` environment variable)
/// wins if it's set, parses as a `u16` and isn't a privileged port (below
/// 1024). Otherwise the configured port.
fn effective_port(config: &Config, env_port: Option<&str>) -> u16 {
    if let Some(raw) = env_port
        && let Ok(port) = raw.parse::<u16>()
        && port >= 1024
    {
        return port;
    }
    config.port
}

const SAMPLE: &str = "\
# demo server
host = example.org
port = 9000

tls_cert = /etc/demo/cert.pem
tls_key = /etc/demo/key.pem
";

fn main() {
    let config = load(SAMPLE);
    println!("{config:?}");
    println!("listening on {}", effective_port(&config, Some("9100")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_pairs() {
        assert_eq!(split_pair("a = b"), Some(("a", "b")));
        assert_eq!(split_pair("  mode=  "), Some(("mode", "")));
        assert_eq!(split_pair("url = a?b=c"), Some(("url", "a?b=c")));
        assert_eq!(split_pair(" = b"), None);
        assert_eq!(split_pair("no equals sign"), None);
    }

    #[test]
    fn tls_needs_both_halves() {
        let both = tls(Some("c.pem"), Some("k.pem"));
        assert_eq!(
            both,
            Some(Tls {
                cert: "c.pem".to_string(),
                key: "k.pem".to_string()
            })
        );
        assert_eq!(tls(Some("c.pem"), None), None);
        assert_eq!(tls(None, Some("k.pem")), None);
        assert_eq!(tls(None, None), None);
    }

    #[test]
    fn loads_a_config() {
        let config = load(SAMPLE);
        assert_eq!(
            config,
            Config {
                host: "example.org".to_string(),
                port: 9000,
                tls: Some(Tls {
                    cert: "/etc/demo/cert.pem".to_string(),
                    key: "/etc/demo/key.pem".to_string(),
                }),
                bad_lines: vec![],
            }
        );
    }

    #[test]
    fn keeps_defaults_and_reports_bad_lines() {
        let text = "port = high\n\nhost\n  # fine\nmode = fast\ntls_cert = c.pem\nport = 70000";
        let config = load(text);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.tls, None);
        assert_eq!(config.bad_lines, [1, 3, 5, 7]);
    }

    #[test]
    fn later_lines_win() {
        let config = load("port = 9000\nport = 9001\nhost = a\nhost = b");
        assert_eq!((config.host.as_str(), config.port), ("b", 9001));
    }

    #[test]
    fn environment_port() {
        let config = load("port = 9000");
        assert_eq!(effective_port(&config, Some("9100")), 9100);
        assert_eq!(effective_port(&config, Some("1024")), 1024);
        assert_eq!(effective_port(&config, Some("80")), 9000);
        assert_eq!(effective_port(&config, Some("lots")), 9000);
        assert_eq!(effective_port(&config, None), 9000);
    }
}

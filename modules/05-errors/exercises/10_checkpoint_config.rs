//! # Checkpoint: a config parser
//!
//! The module in one small parser. The library half, `parse_config`,
//! returns a typed error with line numbers and a `source()` chain, so
//! callers can match on what went wrong. The application half,
//! `load_config`, uses anyhow to say which file was involved. In between:
//! `?`, `ok_or`, `map_err`, defaults. Implement the three functions.
//!
//! Rules: no `unwrap`/`expect`.

use std::num::ParseIntError;
use std::path::Path;

use thiserror::Error;

/// A config file looks like this. `max_connections` defaults to 100 and
/// `timeout_secs` to 30; `host` and `port` are required.
///
/// ```ini
/// # comments start with '#' or ';'
/// [server]
/// host = example.org
/// port = 8080
///
/// [limits]
/// max_connections = 500
/// timeout_secs = 10
/// ```
#[derive(Debug, PartialEq)]
struct Config {
    host: String,
    port: u16,
    max_connections: u32,
    timeout_secs: u64,
}

#[derive(Debug, Error, PartialEq)]
enum ConfigError {
    #[error("line {line}: expected \"[section]\" or \"key = value\"")]
    Syntax { line: usize },
    #[error("line {line}: unknown section [{name}]")]
    UnknownSection { line: usize, name: String },
    #[error("line {line}: \"{key}\" is outside of any section")]
    NoSection { line: usize, key: String },
    #[error("line {line}: unknown key \"{key}\" in [{section}]")]
    UnknownKey {
        line: usize,
        section: String,
        key: String,
    },
    #[error("line {line}: \"{key}\" was already set on line {first}")]
    Duplicate {
        line: usize,
        key: String,
        first: usize,
    },
    #[error("line {line}: invalid value for \"{key}\"")]
    BadValue {
        line: usize,
        key: String,
        source: ParseIntError,
    },
    #[error("missing \"{key}\" in [{section}]")]
    Missing {
        section: &'static str,
        key: &'static str,
    },
}

/// One meaningful line of a config file.
#[derive(Debug, PartialEq)]
enum Line<'a> {
    Section(&'a str),
    Pair(&'a str, &'a str),
}

/// Classifies one trimmed line that isn't blank or a comment. `[<name>]`
/// is a section, the name trimmed and non-empty. `<key> = <value>` is a
/// pair, split at the first '=', both parts trimmed and non-empty.
/// Anything else is a `Syntax` error for `line`.
fn parse_line(line: usize, text: &str) -> Result<Line<'_>, ConfigError> {
    todo!("classify line {line}: {text:?}")
}

/// Parses a config. Lines are numbered from 1 and trimmed; blank lines and
/// lines starting with '#' or ';' are skipped. Lines are checked in order,
/// and the first problem wins:
/// - a section must be `[server]` or `[limits]`, and may appear again;
/// - a pair must follow a section, and use a key known there: `host` and
///   `port` in `[server]`, `max_connections` and `timeout_secs` in
///   `[limits]`;
/// - a key set twice is a `Duplicate`, whatever its value, pointing at
///   the line that set it first;
/// - numbers must parse as their field's type (`BadValue`, with the
///   `ParseIntError` as its `source()`).
///
/// After the last line, a missing `host` or `port` is an error (`host`
/// first), and the limits get their defaults.
fn parse_config(text: &str) -> Result<Config, ConfigError> {
    todo!("parse {text:?}")
}

/// Reads and parses the config file at `path`. Errors get the context
/// `reading <path>` or `parsing <path>`.
fn load_config(path: &Path) -> anyhow::Result<Config> {
    todo!("load {}", path.display())
}

fn main() {
    let text = "[server]\nhost = example.org\nport = 8080\n\n[limits]\ntimeout_secs = 10\n";
    println!("{:?}", parse_config(text));
    for bad in [
        "[server]\nhost = a\nport = 99999",
        "[server]\nport = 1\nport = 2",
    ] {
        if let Err(e) = parse_config(bad) {
            println!("{e}");
        }
    }
    if let Err(e) = load_config(Path::new("no-such-config.ini")) {
        println!("{e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::{fs, io};

    const EXAMPLE: &str = "\
# staging
[server]
host = example.org
port = 8080

[limits]
; tighter than the default
timeout_secs = 10
";

    fn config(host: &str, port: u16, max_connections: u32, timeout_secs: u64) -> Config {
        Config {
            host: host.to_string(),
            port,
            max_connections,
            timeout_secs,
        }
    }

    fn message(text: &str) -> String {
        parse_config(text).unwrap_err().to_string()
    }

    #[test]
    fn classifies_lines() {
        assert_eq!(parse_line(1, "[server]"), Ok(Line::Section("server")));
        assert_eq!(parse_line(1, "[ limits ]"), Ok(Line::Section("limits")));
        assert_eq!(parse_line(1, "port = 8080"), Ok(Line::Pair("port", "8080")));
        assert_eq!(parse_line(1, "a=b=c"), Ok(Line::Pair("a", "b=c")));
        for (i, bad) in ["port 8080", "= 3", "port =", "[server", "[ ]"]
            .iter()
            .enumerate()
        {
            assert_eq!(parse_line(i, bad), Err(ConfigError::Syntax { line: i }));
        }
    }

    #[test]
    fn parses_configs() {
        assert_eq!(
            parse_config(EXAMPLE),
            Ok(config("example.org", 8080, 100, 10))
        );
        let full = "[limits]\nmax_connections=500\n[server]\r\nhost=a\r\nport=1\r\n[limits]\ntimeout_secs=5";
        assert_eq!(parse_config(full), Ok(config("a", 1, 500, 5)));
    }

    #[test]
    fn structure_errors() {
        assert_eq!(
            message("[server]\nport 8080"),
            "line 2: expected \"[section]\" or \"key = value\""
        );
        assert_eq!(
            message("[server]\nhost = a\n\n[database]"),
            "line 4: unknown section [database]"
        );
        assert_eq!(
            message("host = a"),
            "line 1: \"host\" is outside of any section"
        );
        assert_eq!(
            message("[limits]\nport = 80"),
            "line 2: unknown key \"port\" in [limits]"
        );
        assert_eq!(
            message("[server]\nport = 1\nhost = a\n[server]\nport = x"),
            "line 5: \"port\" was already set on line 2"
        );
    }

    #[test]
    fn bad_values_keep_their_cause() {
        let err = parse_config("[server]\nhost = a\nport = 99999").unwrap_err();
        assert!(matches!(err, ConfigError::BadValue { line: 3, .. }));
        assert_eq!(err.to_string(), "line 3: invalid value for \"port\"");
        let cause = err.source().unwrap();
        assert_eq!(cause.to_string(), "number too large to fit in target type");
        assert_eq!(
            message("[server]\nhost = a\nport = 1\n[limits]\ntimeout_secs = -1"),
            "line 5: invalid value for \"timeout_secs\""
        );
    }

    #[test]
    fn missing_keys() {
        assert_eq!(message(""), "missing \"host\" in [server]");
        assert_eq!(
            message("[server]\nhost = a"),
            "missing \"port\" in [server]"
        );
    }

    #[test]
    fn missing_files() {
        let err = load_config(Path::new("no-such-config.ini")).unwrap_err();
        assert_eq!(err.to_string(), "reading no-such-config.ini");
        let cause = err.downcast_ref::<io::Error>().unwrap();
        assert_eq!(cause.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn loads_files() {
        let dir = std::env::temp_dir();
        let good = dir.join(format!("rusty-05-good-{}.ini", std::process::id()));
        let bad = dir.join(format!("rusty-05-bad-{}.ini", std::process::id()));
        fs::write(&good, EXAMPLE).unwrap();
        fs::write(&bad, "[server]\nport = 99999\n").unwrap();
        let loaded = load_config(&good);
        let failed = load_config(&bad);
        fs::remove_file(&good).unwrap();
        fs::remove_file(&bad).unwrap();

        assert_eq!(loaded.unwrap(), config("example.org", 8080, 100, 10));
        let err = failed.unwrap_err();
        let expected = format!(
            "parsing {}: line 2: invalid value for \"port\": number too large to fit in target type",
            bad.display()
        );
        assert_eq!(format!("{err:#}"), expected);
        assert!(matches!(
            err.downcast_ref::<ConfigError>(),
            Some(ConfigError::BadValue { line: 2, .. })
        ));
    }
}

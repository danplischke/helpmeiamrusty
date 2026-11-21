//! # Checkpoint: a config parser (reference solution)
//!
//! - Library half, typed errors: every variant carries what a caller
//!   needs (line, key, first line) and `BadValue` keeps the
//!   `ParseIntError` as its `source()` instead of repeating its text.
//! - Matching on `(section, key)` turns "known key in this section" into
//!   the match itself; the `_` arm is the error. One `bad` closure builds
//!   the `BadValue` for whichever number arm runs, via `map_err(bad)?`.
//! - The duplicate check comes before the value is parsed, so a repeated
//!   key is reported as such even when its second value is also bad.
//! - `Option` fields collect the values; at the end `ok_or(..)?` makes the
//!   required ones errors and `unwrap_or(..)` gives the rest defaults.
//! - Application half: `load_config` adds the file name with anyhow and
//!   changes nothing else. A caller can still
//!   `downcast_ref::<ConfigError>()` to react to a specific problem, and
//!   `{:#}` prints the whole story on one line.

use std::fs;
use std::num::ParseIntError;
use std::path::Path;

use anyhow::Context;
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
    if let Some(name) = text.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let name = name.trim();
        if !name.is_empty() {
            return Ok(Line::Section(name));
        }
    } else if let Some((key, value)) = text.split_once('=') {
        let (key, value) = (key.trim(), value.trim());
        if !key.is_empty() && !value.is_empty() {
            return Ok(Line::Pair(key, value));
        }
    }
    Err(ConfigError::Syntax { line })
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
    let mut section = None;
    let mut first_set: Vec<(&str, &str, usize)> = Vec::new();
    let mut host = None;
    let mut port = None;
    let mut max_connections = None;
    let mut timeout_secs = None;

    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with(['#', ';']) {
            continue;
        }
        let (key, value) = match parse_line(line, raw)? {
            Line::Section(name @ ("server" | "limits")) => {
                section = Some(name);
                continue;
            }
            Line::Section(name) => {
                let name = name.to_string();
                return Err(ConfigError::UnknownSection { line, name });
            }
            Line::Pair(key, value) => (key, value),
        };
        let section = section.ok_or_else(|| ConfigError::NoSection {
            line,
            key: key.to_string(),
        })?;
        if let Some(&(.., first)) = first_set
            .iter()
            .find(|&&(s, k, _)| (s, k) == (section, key))
        {
            let key = key.to_string();
            return Err(ConfigError::Duplicate { line, key, first });
        }
        first_set.push((section, key, line));

        let bad = |source| ConfigError::BadValue {
            line,
            key: key.to_string(),
            source,
        };
        match (section, key) {
            ("server", "host") => host = Some(value.to_string()),
            ("server", "port") => port = Some(value.parse().map_err(bad)?),
            ("limits", "max_connections") => max_connections = Some(value.parse().map_err(bad)?),
            ("limits", "timeout_secs") => timeout_secs = Some(value.parse().map_err(bad)?),
            _ => {
                let (section, key) = (section.to_string(), key.to_string());
                return Err(ConfigError::UnknownKey { line, section, key });
            }
        }
    }

    let missing = |key| ConfigError::Missing {
        section: "server",
        key,
    };
    Ok(Config {
        host: host.ok_or(missing("host"))?,
        port: port.ok_or(missing("port"))?,
        max_connections: max_connections.unwrap_or(100),
        timeout_secs: timeout_secs.unwrap_or(30),
    })
}

/// Reads and parses the config file at `path`. Errors get the context
/// `reading <path>` or `parsing <path>`.
fn load_config(path: &Path) -> anyhow::Result<Config> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    parse_config(&text).with_context(|| format!("parsing {}", path.display()))
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

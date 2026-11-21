//! # From `match` ladders to `?`
//!
//! This parser passes every error along by hand: a `match` per step, each
//! with its own `return Err(..)`. It works and the tests pass, but clippy
//! names several of the patterns. Rewrite it until clippy is quiet, then
//! finish the job where clippy says nothing: a `match` that wraps an error
//! is `.map_err(..)?`, one that turns `None` into an error is `.ok_or(..)?`,
//! and one that falls back to a value is `unwrap_or`.

use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, PartialEq)]
enum ReadingError {
    MissingField(&'static str),
    BadStation(ParseIntError),
    BadTemperature(ParseFloatError),
}

#[derive(Debug, PartialEq)]
struct Reading {
    station: u32,
    celsius: f64,
}

/// The value of the first `name=value` field among the space-separated
/// fields of `line`.
fn field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    for part in line.split_whitespace() {
        match part.split_once('=') {
            Some((key, value)) => {
                if key == name {
                    return Some(value);
                }
            }
            None => {}
        }
    }
    None
}

/// Parses a temperature like "21.5" or "21.5C" (one optional trailing C).
fn parse_celsius(s: &str) -> Result<f64, ReadingError> {
    let number = match s.strip_suffix('C') {
        Some(number) => number,
        None => s,
    };
    number.parse().map_err(|e| ReadingError::BadTemperature(e))
}

/// Parses "station=<id> temp=<celsius>". The fields may come in any order,
/// and other fields are ignored.
fn parse_reading(line: &str) -> Result<Reading, ReadingError> {
    let station = match field(line, "station") {
        Some(value) => match value.parse() {
            Ok(id) => id,
            Err(e) => return Err(ReadingError::BadStation(e)),
        },
        None => return Err(ReadingError::MissingField("station")),
    };
    let celsius = match field(line, "temp") {
        Some(value) => match parse_celsius(value) {
            Ok(celsius) => celsius,
            Err(e) => return Err(e),
        },
        None => return Err(ReadingError::MissingField("temp")),
    };
    Ok(Reading { station, celsius })
}

/// Just the station id, for routing: `None` if it's missing or invalid.
fn station_id(line: &str) -> Option<u32> {
    let value = match field(line, "station") {
        Some(value) => value,
        None => return None,
    };
    match value.parse() {
        Ok(id) => Some(id),
        Err(_) => None,
    }
}

/// Parses one reading per line, stopping at the first bad one. Blank lines
/// are skipped.
fn parse_log(text: &str) -> Result<Vec<Reading>, ReadingError> {
    let mut readings = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match parse_reading(line) {
            Ok(reading) => readings.push(reading),
            Err(e) => return Err(e),
        }
    }
    Ok(readings)
}

/// The mean temperature of a log, or `None` for a log without readings.
fn mean_celsius(text: &str) -> Result<Option<f64>, ReadingError> {
    let readings = match parse_log(text) {
        Ok(readings) => readings,
        Err(e) => return Err(e),
    };
    if readings.is_empty() {
        return Ok(None);
    }
    let total: f64 = readings.iter().map(|r| r.celsius).sum();
    Ok(Some(total / readings.len() as f64))
}

fn main() {
    println!("{:?}", parse_reading("station=7 temp=-3.5C wind=12"));
    println!("{:?}", parse_reading("temp=20"));
    println!("{:?}", station_id("temp=20 station=12"));
    println!(
        "{:?}",
        mean_celsius("station=1 temp=20\nstation=2 temp=22C\n")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields() {
        assert_eq!(field("a=1 b=2", "b"), Some("2"));
        assert_eq!(field("a=1 noise b=2 b=3", "b"), Some("2"));
        assert_eq!(field("ab=1", "a"), None);
        assert_eq!(field("", "a"), None);
    }

    #[test]
    fn temperatures() {
        assert_eq!(parse_celsius("21.5"), Ok(21.5));
        assert_eq!(parse_celsius("-3C"), Ok(-3.0));
        assert!(matches!(
            parse_celsius("21.5CC"),
            Err(ReadingError::BadTemperature(_))
        ));
    }

    #[test]
    fn readings() {
        assert_eq!(
            parse_reading("temp=-3.5C wind=12 station=7"),
            Ok(Reading {
                station: 7,
                celsius: -3.5
            })
        );
        assert_eq!(
            parse_reading("temp=20"),
            Err(ReadingError::MissingField("station"))
        );
        assert_eq!(
            parse_reading("station=7"),
            Err(ReadingError::MissingField("temp"))
        );
        assert!(matches!(
            parse_reading("station=seven temp=1"),
            Err(ReadingError::BadStation(_))
        ));
        assert!(matches!(
            parse_reading("station=7 temp=warm"),
            Err(ReadingError::BadTemperature(_))
        ));
    }

    #[test]
    fn station_ids() {
        assert_eq!(station_id("temp=1 station=12"), Some(12));
        assert_eq!(station_id("station=x"), None);
        assert_eq!(station_id("temp=1"), None);
    }

    #[test]
    fn logs() {
        let log = "station=1 temp=20\n\nstation=2 temp=23C\n";
        assert_eq!(parse_log(log).map(|r| r.len()), Ok(2));
        assert_eq!(mean_celsius(log), Ok(Some(21.5)));
        assert_eq!(mean_celsius(""), Ok(None));
        let bad = "station=1 temp=20\nstation=2\nstation=x temp=1";
        assert_eq!(parse_log(bad), Err(ReadingError::MissingField("temp")));
        assert_eq!(mean_celsius(bad), Err(ReadingError::MissingField("temp")));
    }
}

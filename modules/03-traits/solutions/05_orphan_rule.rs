//! # Blanket impls, the orphan rule and newtypes (reference solution)
//!
//! - The orphan rule: in `impl Trait for Type`, the trait or the type must
//!   be local. A local type as a generic *argument* (`Vec<Reading>`) doesn't
//!   make the type local. It only counts in the trait's parameters:
//!   `impl From<Reading> for f64` is allowed, because `From<Reading>` can't
//!   be implemented by any other crate.
//! - The rule exists so that two crates can never write conflicting impls,
//!   and so that adding an impl upstream doesn't break you.
//! - A type alias is only a second name. A newtype is a real type, free at
//!   runtime, but it hides the inner type's methods: reach them through
//!   `.0`, or forward the ones you want. (`Deref` to the inner type also
//!   works, but leaks its whole API.)
//! - A blanket impl covers every type that meets its bounds, including
//!   foreign ones like `i32`. Stable Rust has no specialization, so you
//!   can't add a separate impl for one of those types (E0119).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Reading {
    sensor: &'static str,
    celsius: f64,
}

/// "kitchen 21.5°C"
impl fmt::Display for Reading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}°C", self.sensor, self.celsius)
    }
}

/// Lets a reading stand in for its temperature: `f64::from(reading)`.
impl From<Reading> for f64 {
    fn from(reading: Reading) -> f64 {
        reading.celsius
    }
}

/// Readings in the order they were taken.
#[derive(Debug, Clone, PartialEq)]
struct Series(Vec<Reading>);

/// One reading per line.
impl fmt::Display for Series {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, reading) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{reading}")?;
        }
        Ok(())
    }
}

impl Series {
    /// The average temperature, or `None` if there are no readings.
    fn mean(&self) -> Option<f64> {
        if self.0.is_empty() {
            return None;
        }
        let total: f64 = self.0.iter().map(|&reading| f64::from(reading)).sum();
        Some(total / self.0.len() as f64)
    }

    /// The readings from one sensor, in order, as a new series.
    fn only(&self, sensor: &str) -> Series {
        Series(
            self.0
                .iter()
                .filter(|reading| reading.sensor == sensor)
                .copied()
                .collect(),
        )
    }
}

/// A one-line summary, for logs.
trait Summary {
    fn summary(&self) -> String;
}

/// Everything that can be displayed gets a summary: the first line of its
/// display form, plus " (+N more)" if there were N more lines.
impl<T: fmt::Display> Summary for T {
    fn summary(&self) -> String {
        let text = self.to_string();
        let mut lines = text.lines();
        let first = lines.next().unwrap_or_default();
        match lines.count() {
            0 => first.to_string(),
            more => format!("{first} (+{more} more)"),
        }
    }
}

fn main() {
    let series = Series(vec![
        Reading {
            sensor: "kitchen",
            celsius: 21.5,
        },
        Reading {
            sensor: "garage",
            celsius: 12.0,
        },
    ]);
    println!("{series}");
    println!("{:?}", series.only("garage").mean());
    println!("{}", series.summary());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(sensor: &'static str, celsius: f64) -> Reading {
        Reading { sensor, celsius }
    }

    fn sample() -> Series {
        Series(vec![
            reading("kitchen", 21.5),
            reading("garage", 12.0),
            reading("kitchen", 23.5),
        ])
    }

    #[test]
    fn displays_one_reading_per_line() {
        assert_eq!(
            sample().to_string(),
            "kitchen 21.5°C\ngarage 12°C\nkitchen 23.5°C"
        );
        assert_eq!(Series(Vec::new()).to_string(), "");
    }

    #[test]
    fn averages_and_filters() {
        let series = sample();
        assert_eq!(series.mean(), Some(19.0));
        let kitchen = series.only("kitchen");
        assert_eq!(kitchen.0.len(), 2);
        assert_eq!(kitchen.mean(), Some(22.5));
        assert_eq!(series.only("attic").mean(), None);
    }

    #[test]
    fn readings_convert_to_f64() {
        let celsius: f64 = reading("attic", -3.5).into();
        assert_eq!(celsius, -3.5);
    }

    #[test]
    fn everything_displayable_has_a_summary() {
        assert_eq!(42.summary(), "42");
        assert_eq!("plain text".summary(), "plain text");
        assert_eq!(reading("garage", 12.0).summary(), "garage 12°C");
        assert_eq!(sample().summary(), "kitchen 21.5°C (+2 more)");
    }
}

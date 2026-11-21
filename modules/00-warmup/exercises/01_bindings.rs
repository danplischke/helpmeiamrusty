//! # Bindings, mutability, shadowing
//!
//! A small temperature log that doesn't compile. The logic is right; the
//! *declarations* are wrong. Fix them until it builds and the tests pass.
//!
//! Refresh: `let` bindings are immutable unless marked `mut`, a `const` needs
//! a value of exactly its declared type, and comparing `&f64` with `f64`
//! is a type error (no implicit derefs in comparisons).

const FREEZING_F: f64 = 32;

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - FREEZING_F) * 5.0 / 9.0
}

/// Returns the (lowest, highest) reading.
fn summarize(readings: &[f64]) -> (f64, f64) {
    let lowest = f64::INFINITY;
    let highest = f64::NEG_INFINITY;
    for r in readings {
        if r < lowest {
            lowest = r;
        }
        if r > highest {
            highest = r;
        }
    }
    (lowest, highest)
}

/// Parses a reading like " 50°F " into degrees Celsius.
fn parse_reading(raw: &str) -> f64 {
    // Shadowing: each `let raw` is a new binding. The last one is an f64.
    let raw = raw.trim();
    let raw = raw.strip_suffix("°F").unwrap_or(raw);
    let raw: f64 = raw.parse().expect("not a number");
    fahrenheit_to_celsius(raw)
}

fn main() {
    let readings: Vec<f64> = ["50°F", "68°F", "32°F"]
        .into_iter()
        .map(parse_reading)
        .collect();
    let (low, high) = summarize(&readings);
    println!("low {low:.1}°C, high {high:.1}°C");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts() {
        assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn summarizes() {
        assert_eq!(summarize(&[3.0, -1.5, 7.25]), (-1.5, 7.25));
    }

    #[test]
    fn parses() {
        assert!((parse_reading(" 50°F ") - 10.0).abs() < 1e-9);
    }
}

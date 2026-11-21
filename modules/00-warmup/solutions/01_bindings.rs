//! # Bindings, mutability, shadowing (reference solution)
//!
//! - `32` is an integer literal; a `const` of type `f64` needs `32.0`.
//! - `for &r in readings` destructures each `&f64` into an `f64`. That works
//!   because `f64` is `Copy`. (`for r in readings` plus `*r` is equivalent.)
//! - `f64::min` / `f64::max` replace the if-chains. Floats aren't `Ord`
//!   (NaN compares unequal to everything), which is why `iter().max()`
//!   doesn't work on them, but these methods do.

const FREEZING_F: f64 = 32.0;

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - FREEZING_F) * 5.0 / 9.0
}

/// Returns the (lowest, highest) reading.
fn summarize(readings: &[f64]) -> (f64, f64) {
    let mut lowest = f64::INFINITY;
    let mut highest = f64::NEG_INFINITY;
    for &r in readings {
        lowest = lowest.min(r);
        highest = highest.max(r);
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

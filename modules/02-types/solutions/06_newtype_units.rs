//! # Newtypes for units (reference solution)
//!
//! - A tuple struct around one field costs nothing at runtime: `Feet` is
//!   the same size as `f64`. What it buys is that `Feet` and `Meters` no
//!   longer mix by accident. Both bugs became compile errors (E0369
//!   "cannot subtract `Meters` from `Feet`", E0308) as soon as the
//!   signatures said what they meant.
//! - Implement only the operations that make physical sense. `Meters -
//!   Meters` is a length; there is no `Feet - Meters`, so the caller has to
//!   convert first. Every `.0` is a place where units can mix again, so
//!   keep them inside the type's own impls.
//! - `From` gives you `Into` for free: `let m: Meters = feet.into();`.
//! - `Iterator::max` needs `Ord`, which `f64` (because of NaN) and a newtype
//!   around it don't have. `max_by(|a, b| a.0.total_cmp(&b.0))` supplies a
//!   total order.
//! - Crates such as `uom` take this all the way, with dimensions checked
//!   at compile time.

use std::ops::Sub;

const METERS_PER_FOOT: f64 = 0.3048;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Feet(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

impl From<Feet> for Meters {
    fn from(feet: Feet) -> Self {
        Meters(feet.0 * METERS_PER_FOOT)
    }
}

impl From<Meters> for Feet {
    fn from(meters: Meters) -> Self {
        Feet(meters.0 / METERS_PER_FOOT)
    }
}

impl Sub for Meters {
    type Output = Meters;

    fn sub(self, other: Meters) -> Meters {
        Meters(self.0 - other.0)
    }
}

/// The height above the terrain, in meters.
fn clearance(altitude: Feet, terrain: Meters) -> Meters {
    Meters::from(altitude) - terrain
}

/// The lowest altitude that clears `terrain` by at least `margin`,
/// rounded up to a multiple of 100 ft.
fn minimum_altitude(terrain: Meters, margin: Feet) -> Feet {
    let needed = Feet::from(terrain).0 + margin.0;
    Feet((needed / 100.0).ceil() * 100.0)
}

/// The highest point of an elevation profile, or `None` if it's empty.
fn highest_point(profile: &[Meters]) -> Option<Meters> {
    profile.iter().copied().max_by(|a, b| a.0.total_cmp(&b.0))
}

/// Whether flying at `altitude` clears every point of `profile` by at
/// least `margin`. An empty profile is safe.
fn is_safe(altitude: Feet, profile: &[Meters], margin: Feet) -> bool {
    match highest_point(profile) {
        Some(top) => clearance(altitude, top) >= Meters::from(margin),
        None => true,
    }
}

fn main() {
    let profile = [Meters(412.0), Meters(1180.5), Meters(960.0)];
    println!("highest point: {:?}", highest_point(&profile));
    println!(
        "minimum altitude: {:?}",
        minimum_altitude(Meters(1180.5), Feet(1000.0))
    );
    println!(
        "safe at 5000 ft? {}",
        is_safe(Feet(5000.0), &profile, Feet(1000.0))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn converts_both_ways() {
        assert!(close(Meters::from(Feet(1000.0)).0, 304.8));
        assert!(close(Feet::from(Meters(304.8)).0, 1000.0));
        let meters: Meters = Feet(10.0).into();
        assert!(close(meters.0, 3.048));
    }

    #[test]
    fn clearance_is_in_meters() {
        let above = clearance(Feet(10_000.0), Meters(1000.0));
        assert!(close(above.0, 2048.0), "got {above:?}");
    }

    #[test]
    fn minimum_altitude_rounds_up() {
        assert_eq!(minimum_altitude(Meters(1000.0), Feet(1000.0)), Feet(4300.0));
        assert_eq!(minimum_altitude(Meters(0.0), Feet(500.0)), Feet(500.0));
    }

    #[test]
    fn finds_the_highest_point() {
        let profile = [Meters(120.0), Meters(845.5), Meters(300.0)];
        assert_eq!(highest_point(&profile), Some(Meters(845.5)));
        assert_eq!(highest_point(&[]), None);
    }

    #[test]
    fn safety_compares_like_with_like() {
        let profile = [Meters(800.0), Meters(1200.0)];
        // 5000 ft is 1524 m: 324 m (about 1063 ft) above the highest point.
        assert!(is_safe(Feet(5000.0), &profile, Feet(1000.0)));
        assert!(!is_safe(Feet(5000.0), &profile, Feet(1100.0)));
        assert!(!is_safe(Feet(3000.0), &profile, Feet(0.0)));
        assert!(is_safe(Feet(0.0), &[], Feet(500.0)));
    }
}

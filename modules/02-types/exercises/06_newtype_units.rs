//! # Newtypes for units
//!
//! In this flight planner, altitudes are in feet and terrain heights in
//! meters, both as bare `f64`s, and two functions mix them up unnoticed.
//! Add `struct Feet(f64)` and `struct Meters(f64)` (deriving `Debug`,
//! `Clone`, `Copy`, `PartialEq`, `PartialOrd`) with `From` conversions both
//! ways (1 ft = 0.3048 m exactly), and use them in every signature, as the
//! tests already do. Once the types are in, the compiler finds the bugs.

const METERS_PER_FOOT: f64 = 0.3048;

/// The height above the terrain, in meters.
fn clearance(altitude_ft: f64, terrain_m: f64) -> f64 {
    altitude_ft - terrain_m
}

/// The lowest altitude that clears `terrain_m` by at least `margin_ft`,
/// rounded up to a multiple of 100 ft.
fn minimum_altitude(terrain_m: f64, margin_ft: f64) -> f64 {
    let needed_ft = terrain_m / METERS_PER_FOOT + margin_ft;
    (needed_ft / 100.0).ceil() * 100.0
}

/// The highest point of an elevation profile, or `None` if it's empty.
fn highest_point(profile_m: &[f64]) -> Option<f64> {
    profile_m.iter().copied().reduce(f64::max)
}

/// Whether flying at `altitude_ft` clears every point of `profile_m` by at
/// least `margin_ft`. An empty profile is safe.
fn is_safe(altitude_ft: f64, profile_m: &[f64], margin_ft: f64) -> bool {
    match highest_point(profile_m) {
        Some(top) => clearance(altitude_ft, top) >= margin_ft,
        None => true,
    }
}

fn main() {
    let profile = [412.0, 1180.5, 960.0];
    println!("highest point: {:?} m", highest_point(&profile));
    println!("minimum altitude: {} ft", minimum_altitude(1180.5, 1000.0));
    println!("safe at 5000 ft? {}", is_safe(5000.0, &profile, 1000.0));
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

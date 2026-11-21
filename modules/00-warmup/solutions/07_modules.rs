//! # Modules, paths and visibility (reference solution)
//!
//! - `format` is a sibling of `shapes`, so from inside `shapes` it's
//!   `super::format` (or `crate::geometry::format`).
//! - `pub(crate)` exposes `format` to the whole crate without making it part
//!   of a public API. There's also `pub(super)` and `pub(in some::path)`.
//! - `Circle::new` must be `pub` to be called from outside `shapes`, while
//!   the field stays private. That's how Rust types protect their invariants.
//! - A module can always see its ancestors' private items, which is why the
//!   `tests` module can reach everything through `use super::*`.

mod geometry {
    pub mod shapes {
        #[derive(Debug)]
        pub struct Circle {
            radius: f64,
        }

        impl Circle {
            pub fn new(radius: f64) -> Self {
                Self { radius }
            }

            pub fn area(&self) -> f64 {
                std::f64::consts::PI * self.radius * self.radius
            }
        }

        pub fn describe(circle: &Circle) -> String {
            format!("circle of area {}", super::format::round2(circle.area()))
        }
    }

    pub(crate) mod format {
        pub fn round2(x: f64) -> f64 {
            (x * 100.0).round() / 100.0
        }
    }
}

use geometry::shapes::{self, Circle};

fn main() {
    let circle = Circle::new(2.0);
    println!("{}", shapes::describe(&circle));
    println!("{}", geometry::format::round2(1.23456));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_describes() {
        let circle = Circle::new(1.0);
        assert_eq!(shapes::describe(&circle), "circle of area 3.14");
    }

    #[test]
    fn rounding_is_reusable() {
        assert_eq!(geometry::format::round2(1.23456).to_string(), "1.23");
    }
}

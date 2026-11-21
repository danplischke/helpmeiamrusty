//! # Modules, paths and visibility
//!
//! Items are private to their module unless marked `pub`. Paths are
//! relative to the current module: `super::` goes up a level, `crate::`
//! starts at the crate root. Fix the paths and the visibility so this
//! compiles. Keep `radius` private: outside code must use the constructor.

mod geometry {
    pub mod shapes {
        #[derive(Debug)]
        pub struct Circle {
            radius: f64,
        }

        impl Circle {
            fn new(radius: f64) -> Self {
                Self { radius }
            }

            pub fn area(&self) -> f64 {
                std::f64::consts::PI * self.radius * self.radius
            }
        }

        pub fn describe(circle: &Circle) -> String {
            format!("circle of area {}", format::round2(circle.area()))
        }
    }

    mod format {
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

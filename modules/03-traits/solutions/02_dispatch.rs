//! # Static and dynamic dispatch (reference solution)
//!
//! - `-> impl Shape` hides one concrete type; it can't be "a `Circle` or a
//!   `Rect`". To return either, box it (`Box<dyn Shape>`). Another design
//!   is an enum that implements `Shape` by matching on itself: no
//!   allocation, but a closed set of shapes.
//! - `[dyn Shape]` can't exist: slice elements need a size. A slice of
//!   `Box<dyn Shape>` (or `&dyn Shape`) holds fat pointers: data plus vtable.
//! - Generic parameters are implicitly `Sized`. `S: Shape + ?Sized` lets
//!   `describe` take `&dyn Shape` as well as `&Circle`. Each concrete `S` gets
//!   its own monomorphized copy; `S = dyn Shape` goes through the vtable.
//!   `fn describe(shape: &dyn Shape)` would also pass the tests: one copy,
//!   always dynamic.
//! - `vec![Box::new(a), Box::new(b)]` infers the element type from the first
//!   element. Annotate `Vec<Box<dyn Shape>>` so each `Box` is coerced.

use std::f64::consts::PI;

trait Shape {
    fn name(&self) -> &str;
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

struct Rect {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn name(&self) -> &str {
        "circle"
    }

    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}

impl Shape for Rect {
    fn name(&self) -> &str {
        "rect"
    }

    fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// Parses "circle <radius>" or "rect <width> <height>". Anything else is
/// `None`.
fn parse_shape(spec: &str) -> Option<Box<dyn Shape>> {
    let words: Vec<&str> = spec.split_whitespace().collect();
    match words.as_slice() {
        ["circle", radius] => Some(Box::new(Circle {
            radius: radius.parse().ok()?,
        })),
        ["rect", width, height] => Some(Box::new(Rect {
            width: width.parse().ok()?,
            height: height.parse().ok()?,
        })),
        _ => None,
    }
}

/// The sum of all the areas.
fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|shape| shape.area()).sum()
}

/// "circle, area 3.14": the name, then the area to two decimals.
fn describe<S: Shape + ?Sized>(shape: &S) -> String {
    format!("{}, area {:.2}", shape.name(), shape.area())
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rect {
            width: 2.0,
            height: 3.0,
        }),
    ];
    println!("total {:.2}", total_area(&shapes));
    for shape in &shapes {
        println!("{}", describe(shape.as_ref()));
    }
    println!("{:?}", parse_shape("rect 1 2").map(|shape| shape.area()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_either_kind() {
        let circle = parse_shape("circle 2").unwrap();
        assert_eq!(circle.name(), "circle");
        let rect = parse_shape("rect 2 3.5").unwrap();
        assert_eq!(rect.area(), 7.0);
        for bad in [
            "",
            "circle",
            "rect 2",
            "hexagon 1",
            "circle x",
            "rect 1 2 3",
        ] {
            assert!(parse_shape(bad).is_none(), "{bad:?} should not parse");
        }
    }

    #[test]
    fn total_area_of_mixed_shapes() {
        let shapes: Vec<Box<dyn Shape>> = ["circle 1", "rect 2 3"]
            .into_iter()
            .filter_map(parse_shape)
            .collect();
        assert!((total_area(&shapes) - (PI + 6.0)).abs() < 1e-9);
        assert_eq!(total_area(&[]), 0.0);
    }

    #[test]
    fn describes_concrete_and_dynamic_shapes() {
        let rect = Rect {
            width: 2.0,
            height: 1.5,
        };
        assert_eq!(describe(&rect), "rect, area 3.00");
        let boxed: Box<dyn Shape> = Box::new(Circle { radius: 1.0 });
        assert_eq!(describe(boxed.as_ref()), "circle, area 3.14");
        let borrowed: &dyn Shape = &Circle { radius: 2.0 };
        assert_eq!(describe(borrowed), "circle, area 12.57");
    }
}

//! # Structs, impl blocks and derives
//!
//! A semantic version, `major.minor.patch`. Implement the methods. One test
//! will still fail afterwards, for a reason that has nothing to do with the
//! code inside the functions.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    patch: u32,
    minor: u32,
    major: u32,
}

impl Version {
    /// A constructor: an associated function (no `self`), called as `Version::new`.
    fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            patch,
            minor,
            major,
        }
    }

    /// Parses "1.2.3". Anything else ("1.2", "1.2.3.4", "1.x.3", "") is `None`.
    fn parse(s: &str) -> Option<Self> {
        todo!("parse {s:?}")
    }

    /// 1.4.2 becomes 1.5.0.
    fn bump_minor(&mut self) {
        todo!("bump the minor version of {self:?}")
    }

    /// Takes `self` by value. `Version` is `Copy`, so the caller keeps theirs.
    /// 1.4.2 -> 2.0.0
    fn next_major(self) -> Self {
        todo!("the major version after {self:?}")
    }
}

/// Makes `format!("{v}")` and `v.to_string()` work: "1.2.3".
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f; // only here to silence "unused variable"
        todo!("format {self:?}: `write!(f, ...)` works like `format!`, into `f`")
    }
}

fn main() {
    let mut versions: Vec<Version> = ["1.10.0", "1.9.3", "2.0.0", "0.9.12"]
        .into_iter()
        .filter_map(Version::parse)
        .collect();
    versions.sort();
    for v in &versions {
        println!("{v}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_versions() {
        assert_eq!(Version::parse("1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse("10.0.42"), Some(Version::new(10, 0, 42)));
    }

    #[test]
    fn rejects_invalid_versions() {
        for bad in ["1.2", "1.2.3.4", "1.x.3", "", "1..3", "-1.2.3"] {
            assert_eq!(Version::parse(bad), None, "{bad:?} should not parse");
        }
    }

    #[test]
    fn displays() {
        assert_eq!(Version::new(1, 2, 3).to_string(), "1.2.3");
    }

    #[test]
    fn bumps() {
        let mut v = Version::new(1, 4, 2);
        v.bump_minor();
        assert_eq!(v, Version::new(1, 5, 0));
        assert_eq!(v.next_major(), Version::new(2, 0, 0));
        assert_eq!(v, Version::new(1, 5, 0), "next_major must not change `v`");
    }

    #[test]
    fn orders_like_semver() {
        assert!(Version::new(1, 10, 0) > Version::new(1, 9, 3));
        assert!(Version::new(2, 0, 0) > Version::new(1, 99, 99));
        let mut vs = vec![
            Version::new(1, 0, 1),
            Version::new(0, 2, 0),
            Version::new(1, 0, 0),
        ];
        vs.sort();
        assert_eq!(
            vs,
            [
                Version::new(0, 2, 0),
                Version::new(1, 0, 0),
                Version::new(1, 0, 1)
            ]
        );
    }
}

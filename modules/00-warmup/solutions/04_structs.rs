//! # Structs, impl blocks and derives (reference solution)
//!
//! - The hidden bug: a derived `PartialOrd`/`Ord` compares fields **in
//!   declaration order**. With `patch` declared first, 1.0.1 sorted after
//!   2.0.0. Declaring `major, minor, patch` fixes the ordering.
//! - `parts.next()??`: `next()` gives `Option<Option<u32>>`; the first `?`
//!   handles "no more parts", the second "not a number". Function arguments
//!   are evaluated left to right, so the order of the parts is kept.
//! - `bool::then_some` turns "only if there are no leftover parts" into an
//!   expression.
//! - `next_major(self)` on a `Copy` type copies, so the caller's value stays.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    /// A constructor: an associated function (no `self`), called as `Version::new`.
    fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parses "1.2.3". Anything else ("1.2", "1.2.3.4", "1.x.3", "") is `None`.
    fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split('.').map(|part| part.parse::<u32>().ok());
        let version = Self::new(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }

    /// 1.4.2 becomes 1.5.0.
    fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Takes `self` by value. `Version` is `Copy`, so the caller keeps theirs.
    /// 1.4.2 -> 2.0.0
    fn next_major(self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }
}

/// Makes `format!("{v}")` and `v.to_string()` work: "1.2.3".
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
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

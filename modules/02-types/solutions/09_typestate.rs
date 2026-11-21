//! # Typestate: the wrong order doesn't compile (reference solution)
//!
//! - A struct has to use each type parameter in a field (E0392); the
//!   compiler works out things like variance and `Send` from the fields.
//!   `PhantomData<S>` counts as a use of `S` without storing one, at zero
//!   size. The `E0282 type annotations needed` errors were knock-on
//!   effects of the same problem.
//! - One `impl` block per state is the whole trick: `publish` lives in
//!   `impl Release<Approved>` and nowhere else, so `tested.publish()` was
//!   E0599 ("the method was found for `Release<Approved>`"). Methods every
//!   state shares go in `impl<S> Release<S>`.
//! - Transitions take `self`, so a stale handle is a use-after-move
//!   (E0382). A fallible transition hands the old state back in its `Err`,
//!   the way `test` does, so nothing is lost.
//! - If states carry data (who approved it), store it instead of a marker:
//!   `struct Approved { by: String }` and a field `state: S`.
//! - Typestate fits linear, compile-time-known workflows (builders,
//!   protocols). When the state comes from runtime data, such as a row in a
//!   database, an enum is the better fit.

use std::marker::PhantomData;

/// The states. They hold no data; they only appear as type parameters.
struct Built;
struct Tested;
struct Approved;

struct Release<S> {
    version: String,
    log: Vec<String>,
    state: PhantomData<S>,
}

impl<S> Release<S> {
    fn version(&self) -> &str {
        &self.version
    }

    fn log(&self) -> &[String] {
        &self.log
    }

    /// Moves to state `T`, logging `entry`. Version and log carry over.
    fn into_state<T>(self, entry: String) -> Release<T> {
        let mut log = self.log;
        log.push(entry);
        Release {
            version: self.version,
            log,
            state: PhantomData,
        }
    }
}

impl Release<Built> {
    fn new(version: &str) -> Self {
        Release {
            version: version.to_string(),
            log: vec![format!("built {version}")],
            state: PhantomData,
        }
    }

    /// Records a test run: "tests passed (12/12)" moves the release on to
    /// `Tested`. A failure logs "tests failed (11/12)" and hands the
    /// release back, still `Built`, so it can be tested again.
    fn test(mut self, passed: u32, total: u32) -> Result<Release<Tested>, Release<Built>> {
        if passed == total {
            Ok(self.into_state(format!("tests passed ({passed}/{total})")))
        } else {
            self.log.push(format!("tests failed ({passed}/{total})"));
            Err(self)
        }
    }
}

impl Release<Tested> {
    fn approve(self, reviewer: &str) -> Release<Approved> {
        self.into_state(format!("approved by {reviewer}"))
    }
}

impl Release<Approved> {
    /// Publishes the release and returns its complete log.
    fn publish(self) -> Vec<String> {
        let mut log = self.log;
        log.push(format!("published {}", self.version));
        log
    }
}

/// Builds, tests, approves (by `reviewer`) and publishes a release, and
/// returns its log. If the tests fail, returns the failed release's log as
/// the error.
fn ship(
    version: &str,
    passed: u32,
    total: u32,
    reviewer: &str,
) -> Result<Vec<String>, Vec<String>> {
    let built = Release::new(version);
    let tested = match built.test(passed, total) {
        Ok(tested) => tested,
        Err(failed) => return Err(failed.log),
    };
    Ok(tested.approve(reviewer).publish())
}

fn main() {
    match Release::new("1.4.0").test(11, 12) {
        Ok(tested) => println!("{} is tested", tested.version()),
        Err(built) => println!("{} needs another run: {:?}", built.version(), built.log()),
    }
    println!("{:?}", ship("1.4.0", 12, 12, "ana"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn walks_through_every_state() {
        let built: Release<Built> = Release::new("1.4.0");
        let Ok(tested) = built.test(12, 12) else {
            panic!("all tests passed");
        };
        let tested: Release<Tested> = tested;
        assert_eq!(tested.version(), "1.4.0");
        let approved: Release<Approved> = tested.approve("ana");
        assert_eq!(
            approved.publish(),
            [
                "built 1.4.0",
                "tests passed (12/12)",
                "approved by ana",
                "published 1.4.0",
            ]
        );
    }

    #[test]
    fn failed_tests_hand_the_release_back() {
        let Err(failed) = Release::new("1.4.1").test(11, 12) else {
            panic!("a test failed");
        };
        let failed: Release<Built> = failed;
        assert_eq!(failed.log(), ["built 1.4.1", "tests failed (11/12)"]);
        let Ok(tested) = failed.test(12, 12) else {
            panic!("the second run passed");
        };
        assert_eq!(tested.log().len(), 3);
    }

    #[test]
    fn ships_or_reports_the_failure() {
        let log = ship("2.0.0", 3, 3, "bo").expect("published");
        assert_eq!(
            log,
            [
                "built 2.0.0",
                "tests passed (3/3)",
                "approved by bo",
                "published 2.0.0",
            ]
        );
        let failed = ship("2.0.1", 2, 3, "bo").expect_err("tests failed");
        assert_eq!(failed, ["built 2.0.1", "tests failed (2/3)"]);
    }

    #[test]
    fn states_cost_nothing() {
        let payload = size_of::<String>() + size_of::<Vec<String>>();
        assert_eq!(size_of::<Release<Built>>(), payload);
        assert_eq!(size_of::<Release<Approved>>(), payload);
    }
}

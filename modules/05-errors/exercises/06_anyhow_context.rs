//! # Application errors with anyhow
//!
//! An application mostly *reports* errors rather than matching on them.
//! `anyhow::Error` holds any error plus layers of context: `.context(..)`
//! and `.with_context(|| ..)` add one (on a `Result` *or* an `Option`),
//! `bail!(..)` returns a fresh error, `ensure!(cond, ..)` bails unless
//! `cond` holds. `{}` prints the outermost layer only, `{:#}` the whole
//! chain joined by ": ". `downcast_ref` digs the original error back out.

use std::path::Path;

use anyhow::{Context, Result, bail, ensure};

/// A replica count: a whole number from 1 to 50. Errors, as `{:#}`
/// prints them:
/// - `replicas "lots" is not a number: invalid digit found in string`
/// - `replicas must be between 1 and 50, got 99`
fn parse_replicas(value: &str) -> Result<u32> {
    todo!("parse {value:?}")
}

/// One `<service> = <replicas>` line; whitespace around either side is
/// fine. Without a '=' or with an empty service name, the error is
/// `expected "<service> = <replicas>"`. Otherwise `parse_replicas`'s.
fn parse_entry(line: &str) -> Result<(String, u32)> {
    todo!("parse {line:?}")
}

/// A deploy plan: one entry per line, in file order. Blank lines and lines
/// starting with '#' are skipped. Errors get the context `line <n>`
/// (counting from 1, every line included), so `{:#}` shows e.g.
/// `line 3: expected "<service> = <replicas>"`.
fn parse_plan(text: &str) -> Result<Vec<(String, u32)>> {
    todo!("parse the plan {text:?}")
}

/// The replica count planned for `service`. If it isn't planned, the
/// error is `service "<service>" is not in the plan`.
fn replicas_for(plan: &[(String, u32)], service: &str) -> Result<u32> {
    todo!("look up {service} in {plan:?}")
}

/// Reads and parses the plan file at `path`. Errors get the context
/// `reading <path>` or `parsing <path>`.
fn load_plan(path: &Path) -> Result<Vec<(String, u32)>> {
    todo!("load {}", path.display())
}

/// The exit code for a failed run, from the original error underneath all
/// the context: 66 for an `io::Error` of kind `NotFound`, 65 for a
/// `ParseIntError` (bad data), 1 for anything else.
fn exit_code(err: &anyhow::Error) -> i32 {
    todo!("pick an exit code for {err:#}")
}

fn main() {
    println!("{:?}", parse_plan("# staging\napi = 2\nweb = 4\n"));
    let failures = [
        parse_plan("api = 2\nweb = many"),
        parse_plan("api = 2\nweb = 70"),
        load_plan(Path::new("no-such-plan.txt")),
    ];
    for err in failures.into_iter().filter_map(Result::err) {
        println!("{err} / {err:#} / exit {}", exit_code(&err));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io};

    fn entry(service: &str, replicas: u32) -> (String, u32) {
        (service.to_string(), replicas)
    }

    #[test]
    fn replica_counts() {
        assert_eq!(parse_replicas("3").unwrap(), 3);
        assert_eq!(parse_replicas("50").unwrap(), 50);
        let err = parse_replicas("lots").unwrap_err();
        assert_eq!(
            format!("{err:#}"),
            "replicas \"lots\" is not a number: invalid digit found in string"
        );
        for (value, n) in [("0", 0), ("99", 99)] {
            let err = parse_replicas(value).unwrap_err();
            let expected = format!("replicas must be between 1 and 50, got {n}");
            assert_eq!(format!("{err:#}"), expected);
        }
    }

    #[test]
    fn entries() {
        assert_eq!(parse_entry("api = 3").unwrap(), entry("api", 3));
        assert_eq!(parse_entry("  web=12 ").unwrap(), entry("web", 12));
        for bad in ["api 3", " = 3"] {
            let err = parse_entry(bad).unwrap_err();
            assert_eq!(format!("{err:#}"), "expected \"<service> = <replicas>\"");
        }
        let err = parse_entry("api = -1").unwrap_err();
        assert_eq!(
            format!("{err:#}"),
            "replicas \"-1\" is not a number: invalid digit found in string"
        );
    }

    #[test]
    fn plans() {
        let text = "# production\napi = 3\n\nweb = 12\n";
        assert_eq!(
            parse_plan(text).unwrap(),
            [entry("api", 3), entry("web", 12)]
        );
        assert!(parse_plan("").unwrap().is_empty());
    }

    #[test]
    fn plan_errors_have_line_numbers() {
        let err = parse_plan("api = 3\n\nweb = lots").unwrap_err();
        assert_eq!(err.to_string(), "line 3");
        assert_eq!(
            format!("{err:#}"),
            "line 3: replicas \"lots\" is not a number: invalid digit found in string"
        );
        let err = parse_plan("# api\nweb").unwrap_err();
        assert_eq!(
            format!("{err:#}"),
            "line 2: expected \"<service> = <replicas>\""
        );
    }

    #[test]
    fn looking_up() {
        let plan = parse_plan("api = 3").unwrap();
        assert_eq!(replicas_for(&plan, "api").unwrap(), 3);
        let err = replicas_for(&plan, "db").unwrap_err();
        assert_eq!(format!("{err:#}"), "service \"db\" is not in the plan");
    }

    #[test]
    fn missing_files() {
        let err = load_plan(Path::new("no-such-plan.txt")).unwrap_err();
        assert_eq!(err.to_string(), "reading no-such-plan.txt");
        let cause = err.downcast_ref::<io::Error>().unwrap();
        assert_eq!(cause.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn bad_files() {
        let name = format!("rusty-05-plan-{}.txt", std::process::id());
        let path = std::env::temp_dir().join(name);
        fs::write(&path, "api = 3\nweb = 0\n").unwrap();
        let result = load_plan(&path);
        fs::remove_file(&path).unwrap();
        let expected = format!(
            "parsing {}: line 2: replicas must be between 1 and 50, got 0",
            path.display()
        );
        assert_eq!(format!("{:#}", result.unwrap_err()), expected);
    }

    #[test]
    fn exit_codes() {
        let missing = load_plan(Path::new("no-such-plan.txt")).unwrap_err();
        assert_eq!(exit_code(&missing), 66);
        let bad_number = parse_plan("api = 3\nweb = lots").unwrap_err();
        assert_eq!(exit_code(&bad_number), 65);
        let out_of_range = parse_plan("api = 99").unwrap_err();
        assert_eq!(exit_code(&out_of_range), 1);
        let denied = anyhow::Error::new(io::Error::from(io::ErrorKind::PermissionDenied));
        assert_eq!(exit_code(&denied), 1);
    }
}

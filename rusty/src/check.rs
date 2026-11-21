//! The check pipeline: rules → compile → tests → clippy (with constraints) → run `main`.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::course::{Exercise, Stage};
use crate::ui;
use crate::work;

const RUN_TIMEOUT: Duration = Duration::from_secs(10);

/// Lints silencing which would defeat the checks, whatever the exercise's constraints.
const ALWAYS_GUARDED: &[&str] = &["clippy::all"];

/// A Cargo package that has a bin target for the file being checked.
pub struct Package {
    pub manifest: PathBuf,
    pub target_dir: PathBuf,
}

pub enum Outcome {
    Pass { output: String },
    Fail { stage: Stage, output: String },
}

pub struct Checker<'a> {
    pub cargo: &'a Path,
    pub package: &'a Package,
    pub clippy_conf_root: &'a Path,
}

impl Checker<'_> {
    /// Checks `source` (built as bin target `bin`) against exercise `ex`.
    /// `pristine` is the exercise's original text; its tests must be left intact.
    pub fn check(
        &self,
        ex: &Exercise,
        bin: &str,
        source: &Path,
        pristine: &str,
    ) -> Result<Outcome> {
        let code =
            fs::read_to_string(source).with_context(|| format!("reading {}", source.display()))?;
        if let Some(problem) = rules_problem(ex, &code, pristine) {
            return Ok(Outcome::Fail {
                stage: Stage::Rules,
                output: problem,
            });
        }

        let compile = self.cargo("test", bin).arg("--no-run").output()?;
        if !compile.status.success() {
            return Ok(fail(Stage::Compile, &compile));
        }

        let test = self
            .cargo("test", bin)
            .args(["--", "--color", ui::color_arg()])
            .env("RUST_BACKTRACE", "0")
            .output()?;
        if !test.status.success() {
            return Ok(fail(Stage::Test, &test));
        }

        let mut clippy = self.cargo("clippy", bin);
        clippy.args(["--", "-D", "clippy::all", "-A", "dead_code"]);
        for constraint in &ex.constraints {
            clippy.args(constraint.lint_args());
        }
        match self.clippy_conf(ex)? {
            Some(dir) => clippy.env("CLIPPY_CONF_DIR", dir),
            None => clippy.env_remove("CLIPPY_CONF_DIR"),
        };
        let clippy = clippy.output()?;
        if !clippy.status.success() {
            return Ok(fail(Stage::Clippy, &clippy));
        }

        if !ex.run_main {
            return Ok(Outcome::Pass {
                output: String::new(),
            });
        }
        let build = self.cargo("build", bin).output()?;
        if !build.status.success() {
            return Ok(fail(Stage::Run, &build));
        }
        let exe = self
            .package
            .target_dir
            .join("debug")
            .join(format!("{bin}{}", std::env::consts::EXE_SUFFIX));
        let mut main = Command::new(&exe);
        main.env("RUST_BACKTRACE", "0");
        let run = run_with_timeout(main, RUN_TIMEOUT)?;
        Ok(match run.status {
            Some(status) if status.success() => Outcome::Pass { output: run.stdout },
            Some(_) => Outcome::Fail {
                stage: Stage::Run,
                output: format!("{}{}", run.stdout, run.stderr),
            },
            None => Outcome::Fail {
                stage: Stage::Run,
                output: format!(
                    "{}{}\n`main` was still running after {}s, so it was stopped.",
                    run.stdout,
                    run.stderr,
                    RUN_TIMEOUT.as_secs()
                ),
            },
        })
    }

    fn cargo(&self, subcommand: &str, bin: &str) -> Command {
        let mut cmd = Command::new(self.cargo);
        cmd.arg(subcommand)
            .args(["--quiet", "--color", ui::color_arg()])
            .arg("--manifest-path")
            .arg(&self.package.manifest)
            .arg("--target-dir")
            .arg(&self.package.target_dir)
            .args(["--bin", bin])
            .stdin(Stdio::null());
        cmd
    }

    /// Writes a clippy.toml for constraints that need configuration, returning its directory.
    fn clippy_conf(&self, ex: &Exercise) -> Result<Option<PathBuf>> {
        let configured: Vec<_> = ex
            .constraints
            .iter()
            .filter(|c| !c.disallowed_methods().is_empty())
            .collect();
        if configured.is_empty() {
            return Ok(None);
        }
        let slug = configured
            .iter()
            .map(|c| c.slug())
            .collect::<Vec<_>>()
            .join("+");
        let dir = self.clippy_conf_root.join(slug);
        let mut conf = String::from("disallowed-methods = [\n");
        for (path, reason) in configured.iter().flat_map(|c| c.disallowed_methods()) {
            conf.push_str(&format!(
                "    {{ path = {}, reason = {} }},\n",
                toml::Value::from(*path),
                toml::Value::from(*reason)
            ));
        }
        conf.push_str("]\n");
        fs::create_dir_all(&dir)?;
        work::write_if_changed(&dir.join("clippy.toml"), &conf)?;
        Ok(Some(dir))
    }
}

/// Checks that the tests are untouched and no constraint lint is silenced.
fn rules_problem(ex: &Exercise, code: &str, pristine: &str) -> Option<String> {
    let expected = work::tests_section(pristine).map(work::squash);
    if work::tests_section(code).map(work::squash) != expected {
        return Some(
            "The tests at the bottom of this file were changed or removed.\n\
             They're the spec, so they have to stay as shipped. Undo the edit, or type\n\
             `restore-tests` to put them back (your code above them is kept)."
                .to_string(),
        );
    }
    let guarded = ex
        .constraints
        .iter()
        .flat_map(|c| c.guarded_lints().iter().map(move |l| (*l, c.label())))
        .chain(ALWAYS_GUARDED.iter().map(|l| (*l, "the clippy check")));
    for (lint, what) in guarded {
        let silenced = work::code_section(code).lines().any(|line| {
            (line.contains("allow(") || line.contains("expect(")) && line.contains(lint)
        });
        if silenced {
            return Some(format!(
                "Silencing `{lint}` doesn't count: it enforces {what}.\n\
                 Remove that #[allow]/#[expect] and solve it for real."
            ));
        }
    }
    None
}

fn fail(stage: Stage, out: &Output) -> Outcome {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // Drop Cargo's boilerplate; it points at commands the learner never ran.
    let output = text
        .lines()
        .filter(|l| {
            !l.contains("to rerun pass") && !l.contains("run the command again with --verbose")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Outcome::Fail { stage, output }
}

pub struct RunResult {
    /// `None` if the process was killed for running too long.
    pub status: Option<ExitStatus>,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_with_timeout(mut cmd: Command, timeout: Duration) -> Result<RunResult> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("running {}", cmd.get_program().to_string_lossy()))?;
    let stdout = child.stdout.take().map(read_in_background);
    let stderr = child.stderr.take().map(read_in_background);
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if start.elapsed() >= timeout {
            child.kill().ok();
            child.wait().ok();
            break None;
        }
        thread::sleep(Duration::from_millis(10));
    };
    Ok(RunResult {
        status,
        stdout: join_reader(stdout),
        stderr: join_reader(stderr),
    })
}

fn read_in_background(mut pipe: impl Read + Send + 'static) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        pipe.read_to_end(&mut buf).ok();
        String::from_utf8_lossy(&buf).into_owned()
    })
}

fn join_reader(handle: Option<JoinHandle<String>>) -> String {
    handle.and_then(|h| h.join().ok()).unwrap_or_default()
}

//! `cargo rusty dev verify`: proves the course content is sound. Every stub must fail
//! (at the stage it claims), every reference solution must pass with the stub's exact
//! tests, files must be rustfmt-clean, and every quiz must have exactly one right answer.

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::check::{Checker, Outcome, Package};
use crate::course::{Course, Exercise};
use crate::ctx::Ctx;
use crate::{quiz, ui, work};

pub fn run(ctx: &Ctx, filter: Option<&str>) -> Result<bool> {
    let course = &ctx.course;
    let selected: Vec<&Exercise> = course
        .exercises()
        .filter(|e| filter.is_none_or(|f| e.id.contains(f)))
        .collect();
    // One scratch package per filter, holding only the selected exercises, so a
    // half-written exercise elsewhere (or a parallel verify run) can't interfere.
    let slug: String = filter
        .unwrap_or("all")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let dir = course.scratch_dir().join("verify").join(slug);
    fs::create_dir_all(&dir)?;
    write_manifest(course, &dir, &selected)?;
    let package = Package {
        manifest: dir.join("Cargo.toml"),
        target_dir: dir.join("target"),
    };
    let conf_root = ctx.clippy_conf_root();
    let checker = Checker {
        cargo: &ctx.cargo,
        package: &package,
        clippy_conf_root: &conf_root,
    };

    let mut failures = 0;
    let mut checked = 0;
    for module in &course.modules {
        let module_selected = filter.is_none_or(|f| module.id.contains(f));
        if module_selected && !module.readme().is_file() {
            report(&module.id, &["missing README.md".to_string()]);
            failures += 1;
        }
        for ex in &module.exercises {
            if filter.is_some_and(|f| !ex.id.contains(f)) {
                continue;
            }
            checked += 1;
            let problems = verify_exercise(&checker, course, ex)?;
            failures += usize::from(!problems.is_empty());
            report(&ex.id, &problems);
        }
        if !module_selected {
            continue;
        }
        for (i, q) in module.quiz.iter().enumerate() {
            checked += 1;
            let label = format!("{} quiz Q{}", module.id, i + 1);
            let mut problems = Vec::new();
            let qdir = course
                .scratch_dir()
                .join("verify-quiz")
                .join(&module.id)
                .join(format!("q{:02}", i + 1));
            let verdict = quiz::evaluate(&q.code, &qdir)?;
            let matching = q.choices.iter().filter(|c| verdict.matches(c)).count();
            if matching != 1 {
                problems.push(format!(
                    "{matching} choices match the real behaviour (need exactly 1).\n{}",
                    verdict.describe()
                ));
            }
            if q.choices.len() < 2 {
                problems.push("needs at least two choices".to_string());
            }
            if let Some(drill) = &q.drill
                && course.resolve_exercise(drill).is_err()
            {
                problems.push(format!("drill `{drill}` doesn't name an exercise"));
            }
            failures += usize::from(!problems.is_empty());
            report(&label, &problems);
        }
    }

    println!("{}", ui::rule());
    if failures == 0 {
        println!("{}", ui::green(&format!("All {checked} items verified.")));
    } else {
        println!(
            "{}",
            ui::red(&format!("{failures} of {checked} items have problems."))
        );
    }
    Ok(failures == 0)
}

fn verify_exercise(checker: &Checker<'_>, course: &Course, ex: &Exercise) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for path in [&ex.stub, &ex.solution] {
        if !path.is_file() {
            problems.push(format!("missing {}", course.rel(path).display()));
        }
    }
    if !problems.is_empty() {
        return Ok(problems);
    }
    if ex.hints.len() < 2 {
        problems.push("needs at least two hints".to_string());
    }
    let stub = fs::read_to_string(&ex.stub)?;
    if work::tests_section(&stub).is_none() {
        problems.push("stub has no `#[cfg(test)]` section at the end".to_string());
    }
    for path in [&ex.stub, &ex.solution] {
        if let Some(diff) = rustfmt_problem(path)? {
            problems.push(format!(
                "{} isn't rustfmt-clean:\n{diff}",
                course.rel(path).display()
            ));
        }
    }
    match checker.check(ex, &stub_bin(ex), &ex.stub, &stub)? {
        Outcome::Pass { .. } => problems.push("the stub already passes".to_string()),
        Outcome::Fail { stage, output } => {
            if let Some(expected) = ex.fails_at
                && expected != stage
            {
                problems.push(format!(
                    "the stub fails at `{stage}`, expected `{expected}`:\n{output}"
                ));
            }
        }
    }
    if let Outcome::Fail { stage, output } =
        checker.check(ex, &solution_bin(ex), &ex.solution, &stub)?
    {
        problems.push(format!("the solution fails at `{stage}`:\n{output}"));
    }
    Ok(problems)
}

fn report(label: &str, problems: &[String]) {
    if problems.is_empty() {
        println!("{} {label}", ui::green("✓"));
    } else {
        println!("{} {label}", ui::red("✗"));
        for p in problems {
            println!("{}", ui::indent(p, "    "));
        }
    }
}

fn stub_bin(ex: &Exercise) -> String {
    format!("stub__{}", ex.bin())
}

fn solution_bin(ex: &Exercise) -> String {
    format!("solution__{}", ex.bin())
}

fn write_manifest(course: &Course, dir: &Path, exercises: &[&Exercise]) -> Result<()> {
    let mut manifest = String::from(
        "# Generated by `cargo rusty dev verify`.\n\
         [package]\nname = \"verify\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\
         autobins = false\nautoexamples = false\nautotests = false\nautobenches = false\n\n\
         [lints.rust]\ndead_code = \"allow\"\n\n\
         # Its own workspace, not part of the course's.\n[workspace]\n",
    );
    manifest.push_str(&work::dependencies_section(course)?);
    for ex in exercises {
        manifest.push_str(&work::bin_section(&stub_bin(ex), &ex.stub));
        manifest.push_str(&work::bin_section(&solution_bin(ex), &ex.solution));
    }
    work::write_if_changed(&dir.join("Cargo.toml"), &manifest)?;
    Ok(())
}

fn rustfmt_problem(path: &Path) -> Result<Option<String>> {
    let out = Command::new("rustfmt")
        .args(["--check", "--edition", "2024"])
        .arg(path)
        .output()
        .context("running rustfmt (install it with `rustup component add rustfmt`)")?;
    Ok((!out.status.success()).then(|| {
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }))
}

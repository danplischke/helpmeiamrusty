//! Screens shared by watch mode and the one-shot commands.

use std::fs;

use anyhow::Result;

use crate::check::Outcome;
use crate::course::{Exercise, Stage};
use crate::ctx::Ctx;
use crate::progress::fmt_duration;
use crate::ui;

pub fn header(ctx: &Ctx, ex: &Exercise) {
    let module = ctx.course.module_of(ex);
    let done = module
        .exercises
        .iter()
        .filter(|e| ctx.progress.is_done(&e.id))
        .count();
    println!(
        "{}",
        ui::banner(
            &format!("{} · {}", module.number(), module.title),
            &format!("{done}/{}", module.exercises.len())
        )
    );
    println!(
        "{} {}  {}",
        ui::cyan("▶"),
        ui::bold(&ex.name),
        ui::dim(&format!("{}: {}", ex.kind.label(), ex.kind.blurb()))
    );
    if !ex.constraints.is_empty() {
        let labels: Vec<&str> = ex.constraints.iter().map(|c| c.label()).collect();
        println!(
            "  {}  {}",
            ui::dim("rules"),
            ui::yellow(&labels.join(" · "))
        );
    }
    println!(
        "  {}   {}",
        ui::dim("edit"),
        ui::bold(&ctx.course.rel(&ex.work).display().to_string())
    );
}

/// Prints a check result. Returns true if the exercise passed.
pub fn outcome(ctx: &Ctx, ex: &Exercise, outcome: &Outcome) -> bool {
    println!("{}", ui::rule());
    match outcome {
        Outcome::Pass { output } => {
            if !output.trim().is_empty() {
                println!("{}", ui::dim("Output of main:"));
                println!("{}", output.trim_end());
                println!("{}", ui::rule());
            }
            let stats = ctx.progress.get(&ex.id).cloned().unwrap_or_default();
            let mut facts = vec![format!("in {}", fmt_duration(stats.active_secs))];
            facts.push(plural(stats.failed_checks, "failed check"));
            facts.push(plural(stats.hints, "hint"));
            println!("{} {}", ui::green("✓ Solved"), ui::dim(&facts.join(" · ")));
            true
        }
        Outcome::Fail { stage, output } => {
            println!("{}", output.trim_end());
            println!("{}", ui::rule());
            let message = match stage {
                Stage::Rules => "Rule check failed.",
                Stage::Compile => "Doesn't compile yet.",
                Stage::Test => "Compiles, but some tests fail.",
                Stage::Clippy => "Tests pass. Now satisfy clippy (that's where the rules live).",
                Stage::Run => "Tests and clippy pass, but running `main` failed.",
            };
            println!("{} {}", ui::red("✗"), message);
            false
        }
    }
}

fn plural(n: u32, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

/// Reveals the next hint. Returns false if there were no more.
pub fn next_hint(ctx: &mut Ctx, ex: &Exercise) -> Result<bool> {
    let total = ex.hints.len();
    let shown = ctx.progress.get(&ex.id).map_or(0, |s| s.hints as usize);
    if shown >= total {
        if total == 0 {
            println!("{}", ui::dim("This exercise has no hints."));
        } else {
            for (i, hint) in ex.hints.iter().enumerate() {
                print_hint(i, total, hint);
            }
        }
        println!(
            "{}",
            ui::dim("That's every hint. Type `give up` to see the reference solution.")
        );
        return Ok(false);
    }
    print_hint(shown, total, &ex.hints[shown]);
    ctx.progress.touch(&ex.id);
    ctx.progress.stats_mut(&ex.id).hints += 1;
    ctx.progress.save()?;
    Ok(true)
}

fn print_hint(index: usize, total: usize, hint: &str) {
    println!(
        "\n{}\n{}",
        ui::yellow(&format!("Hint {}/{total}", index + 1)),
        ui::wrap(hint, "  ")
    );
}

pub fn solution(ctx: &Ctx, ex: &Exercise) -> Result<()> {
    let code = fs::read_to_string(&ex.solution)?;
    println!(
        "\n{}",
        ui::banner(
            "Reference solution",
            &ctx.course.rel(&ex.solution).display().to_string()
        )
    );
    println!("{}", code.trim_end());
    println!("{}", ui::rule());
    println!(
        "{}",
        ui::dim(&format!(
            "Compare with yours:  git diff --no-index {} {}",
            ctx.course.rel(&ex.work).display(),
            ctx.course.rel(&ex.solution).display()
        ))
    );
    Ok(())
}

pub fn list(ctx: &Ctx) {
    let total = ctx.course.exercises().count();
    let done = ctx
        .course
        .exercises()
        .filter(|e| ctx.progress.is_done(&e.id))
        .count();
    println!(
        "{}",
        ui::banner(&ctx.course.title, &format!("{done}/{total} exercises"))
    );
    let current = ctx.current().map(|e| e.id.clone());
    for module in &ctx.course.modules {
        let n = module.exercises.len();
        let d = module
            .exercises
            .iter()
            .filter(|e| ctx.progress.is_done(&e.id))
            .count();
        let quiz = match ctx.progress.quizzes.get(&module.id) {
            _ if module.quiz.is_empty() => String::new(),
            Some(q) if !q.skipped => format!("quiz {}/{}", q.correct, q.total),
            _ => "quiz not taken".to_string(),
        };
        println!(
            "\n{} {}  {} {}  {}",
            ui::bold(module.number()),
            ui::bold(&format!("{:<32}", module.title)),
            ui::progress_bar(d, n, 12),
            format_args!("{d:>2}/{n:<2}"),
            ui::dim(&quiz)
        );
        for ex in &module.exercises {
            let stats = ctx.progress.get(&ex.id);
            let name = format!("{:<28}", ex.name);
            let line = match stats {
                _ if current.as_deref() == Some(ex.id.as_str()) => {
                    format!("{} {}", ui::cyan("▶"), ui::bold(&name))
                }
                Some(s) if s.done => {
                    let mark = if s.gave_up {
                        ui::yellow("~")
                    } else {
                        ui::green("✓")
                    };
                    let facts = format!(
                        "{} · {}",
                        fmt_duration(s.active_secs),
                        plural(s.hints, "hint")
                    );
                    format!("{mark} {name} {}", ui::dim(&facts))
                }
                _ => format!("{} {}", ui::dim("·"), ui::dim(&name)),
            };
            println!("   {line}");
        }
    }
}

pub fn notes(ctx: &Ctx, module_id: &str) -> Result<()> {
    if let Some(module) = ctx.course.module(module_id) {
        let path = module.readme();
        println!(
            "{}\n",
            ui::banner(
                "Refresher notes",
                &ctx.course.rel(&path).display().to_string()
            )
        );
        println!("{}", fs::read_to_string(&path)?.trim_end());
    }
    Ok(())
}

pub fn help() {
    let rows = [
        ("(save the file)", "re-check"),
        ("Enter / c", "re-check now (or go on, once solved)"),
        ("h", "next hint"),
        ("s", "reference solution (once solved)"),
        (
            "give up",
            "mark as given up and show the reference solution",
        ),
        ("n / p", "next / previous exercise"),
        ("l", "list all exercises"),
        ("quiz", "take this module's quiz"),
        ("notes", "print this module's refresher notes"),
        ("reset", "restore the exercise to its starting point"),
        (
            "restore-tests",
            "put back the original tests, keeping your code",
        ),
        ("q", "quit (progress is saved)"),
    ];
    println!();
    for (keys, what) in rows {
        println!("  {}  {what}", ui::bold(&format!("{keys:<15}")));
    }
}

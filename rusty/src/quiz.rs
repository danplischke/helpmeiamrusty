//! Predict-the-output quizzes. Every snippet is compiled and run for real, so the
//! "right answer" is whatever the toolchain actually does.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::check::run_with_timeout;
use crate::course::Question;
use crate::ctx::Ctx;
use crate::progress::{QuizStats, now};
use crate::ui::{self, Input};

const SNIPPET_TIMEOUT: Duration = Duration::from_secs(5);

/// What a snippet does when compiled and run.
pub enum Verdict {
    NoCompile(String),
    Panic(String),
    /// Exited with a failure code without panicking, e.g. `main` returned `Err`.
    ErrorExit(String),
    Output(String),
    Hang,
}

impl Verdict {
    /// Choices are matched against stdout, or one of the special answers
    /// `does not compile`, `panics`, `exits with an error`, `runs forever`, `compiles`.
    pub fn matches(&self, choice: &str) -> bool {
        let choice = choice.trim();
        match self {
            Verdict::NoCompile(_) => choice == "does not compile",
            Verdict::Panic(_) => choice == "panics",
            Verdict::ErrorExit(_) => choice == "exits with an error",
            Verdict::Hang => choice == "runs forever",
            Verdict::Output(out) => choice == "compiles" || choice == out.trim(),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Verdict::NoCompile(err) => format!("It doesn't compile:\n{err}"),
            Verdict::Panic(msg) => format!("It compiles, then panics:\n{msg}"),
            Verdict::ErrorExit(msg) => {
                format!("It compiles, runs, and exits with an error:\n{msg}")
            }
            Verdict::Hang => "It never finishes.".to_string(),
            Verdict::Output(out) if out.trim().is_empty() => {
                "It compiles and runs, printing nothing.".to_string()
            }
            Verdict::Output(out) => format!("It compiles and prints:\n{}", out.trim_end()),
        }
    }
}

/// Compiles and runs `code` in `dir` (debug assertions on, like `cargo run`).
pub fn evaluate(code: &str, dir: &Path) -> Result<Verdict> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("main.rs"), code)?;
    let exe = format!("snippet{}", std::env::consts::EXE_SUFFIX);
    let compile = Command::new("rustc")
        .current_dir(dir)
        .args(["--edition", "2024", "--crate-name", "snippet"])
        .args(["-C", "debug-assertions=yes", "-C", "overflow-checks=yes"])
        .args(["-A", "warnings", "--color", "never", "-o", &exe, "main.rs"])
        .output()
        .context("running rustc (is the Rust toolchain installed?)")?;
    if !compile.status.success() {
        return Ok(Verdict::NoCompile(first_error(&String::from_utf8_lossy(
            &compile.stderr,
        ))));
    }
    let mut snippet = Command::new(dir.join(&exe));
    snippet.env("RUST_BACKTRACE", "0");
    let run = run_with_timeout(snippet, SNIPPET_TIMEOUT)?;
    Ok(match run.status {
        None => Verdict::Hang,
        Some(status) if status.success() => Verdict::Output(run.stdout),
        // Rust exits with 101 after a panic; any other failure code is an ordinary error exit.
        Some(status) if status.code() == Some(101) => Verdict::Panic(panic_message(&run.stderr)),
        Some(_) => Verdict::ErrorExit(
            format!("{}{}", run.stdout, run.stderr)
                .trim_end()
                .to_string(),
        ),
    })
}

fn first_error(stderr: &str) -> String {
    stderr
        .split("\n\n")
        .find(|block| block.trim_start().starts_with("error"))
        .unwrap_or(stderr)
        .trim_end()
        .to_string()
}

fn panic_message(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.starts_with("note:"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn letter(i: usize) -> char {
    char::from(b'a' + u8::try_from(i).unwrap_or(25))
}

fn parse_choice(answer: &str, count: usize) -> Option<usize> {
    let mut chars = answer.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return None;
    };
    let index = (c as usize).checked_sub('a' as usize)?;
    (index < count).then_some(index)
}

/// Runs a module's quiz interactively and records the score.
pub fn run(ctx: &mut Ctx, module_id: &str, input: &Input) -> Result<()> {
    let Some(module) = ctx.course.module(module_id) else {
        bail!("unknown module {module_id}");
    };
    let questions = module.quiz.clone();
    let title = format!("Quiz · {} {}", module.number(), module.title);
    if questions.is_empty() {
        println!("Module {} has no quiz yet.", module.id);
        return Ok(());
    }

    // Compile every snippet in the background while the learner reads.
    let dir = ctx.course.scratch_dir().join("quiz").join(module_id);
    let (tx, rx) = mpsc::channel();
    let snippets: Vec<String> = questions.iter().map(|q| q.code.clone()).collect();
    thread::spawn(move || {
        for (i, code) in snippets.iter().enumerate() {
            let verdict = evaluate(code, &dir.join(format!("q{:02}", i + 1)));
            if tx.send((i, verdict)).is_err() {
                return;
            }
        }
    });
    let mut verdicts: Vec<Option<Verdict>> = questions.iter().map(|_| None).collect();

    let total = questions.len();
    let mut correct = 0;
    let mut missed = Vec::new();
    let mut answered = 0;
    for (i, q) in questions.iter().enumerate() {
        ui::clear_screen();
        println!("{}\n", ui::banner(&title, &format!("{}/{total}", i + 1)));
        show_question(q);
        let answer = loop {
            let Some(line) = input.ask(&format!("\n{} ", ui::cyan("answer ›"))) else {
                return Ok(());
            };
            let line = line.trim().to_lowercase();
            if line == "q" {
                return finish(ctx, module_id, correct, answered, missed);
            }
            match parse_choice(&line, q.choices.len()) {
                Some(index) => break index,
                None => println!(
                    "{}",
                    ui::dim(&format!(
                        "Type a letter from a to {} (or q to stop the quiz).",
                        letter(q.choices.len() - 1)
                    ))
                ),
            }
        };

        while verdicts[i].is_none() {
            let (j, verdict) = rx
                .recv()
                .context("the quiz evaluator stopped unexpectedly")?;
            verdicts[j] = Some(verdict?);
        }
        let verdict = verdicts[i].as_ref().context("verdict was just stored")?;
        let right: Vec<usize> = (0..q.choices.len())
            .filter(|&k| verdict.matches(&q.choices[k]))
            .collect();

        println!();
        if right.is_empty() {
            println!(
                "{}",
                ui::yellow("⚠ None of the choices matches what really happened (a course bug).")
            );
        } else {
            answered += 1;
            if right.contains(&answer) {
                correct += 1;
                println!("{}", ui::green("✓ Correct."));
            } else {
                missed.push(u32::try_from(i + 1).unwrap_or(u32::MAX));
                let k = right[0];
                println!(
                    "{} The answer is {}) {}",
                    ui::red("✗ Not quite."),
                    letter(k),
                    q.choices[k]
                );
            }
        }
        println!("\n{}", ui::indent(&verdict.describe(), "  "));
        println!("\n{}\n{}", ui::bold("Why"), ui::wrap(&q.explain, "  "));
        if let Some(drill) = &q.drill {
            println!("\n  {} exercise {drill}", ui::dim("Drilled in"));
        }
        let more = i + 1 < total;
        let prompt = if more {
            "Enter for the next question · q to stop "
        } else {
            "Enter to see your score "
        };
        let reply = input.ask(&format!("\n{}", ui::dim(prompt)));
        if reply.is_none() || (more && reply.is_some_and(|r| r.trim() == "q")) {
            break;
        }
    }
    finish(ctx, module_id, correct, answered, missed)
}

fn show_question(q: &Question) {
    println!("{}\n", ui::bold(&q.prompt));
    println!("{}", ui::indent(q.code.trim_end(), "    "));
    println!();
    for (k, choice) in q.choices.iter().enumerate() {
        println!("  {}) {choice}", ui::bold(&letter(k).to_string()));
    }
}

fn finish(
    ctx: &mut Ctx,
    module_id: &str,
    correct: u32,
    answered: u32,
    missed: Vec<u32>,
) -> Result<()> {
    if answered == 0 {
        return Ok(());
    }
    ui::clear_screen();
    println!(
        "{}",
        ui::banner("Quiz done", &format!("{correct}/{answered}"))
    );
    if missed.is_empty() {
        println!("\nNo blind spots here. The exercises should go quickly.");
    } else {
        let list = missed
            .iter()
            .map(|m| format!("Q{m}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("\nMissed {list}. The exercises that drill those are worth extra care.");
        if let Some(module) = ctx.course.module(module_id) {
            println!(
                "Refresher notes: {}",
                ui::bold(&ctx.course.rel(&module.readme()).display().to_string())
            );
        }
    }
    ctx.progress.quizzes.insert(
        module_id.to_string(),
        QuizStats {
            correct,
            total: answered,
            taken_at: now(),
            missed,
            skipped: false,
        },
    );
    ctx.progress.save()
}

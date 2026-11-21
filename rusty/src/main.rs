//! `cargo rusty`: the runner for the helpmeiamrusty Rust refresher course.

mod check;
mod course;
mod ctx;
mod progress;
mod quiz;
mod screens;
mod ui;
mod verify;
mod watch;
mod work;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::check::Outcome;
use crate::ctx::Ctx;

#[derive(Parser)]
#[command(
    name = "cargo rusty",
    bin_name = "cargo rusty",
    version,
    about = "An interactive Rust refresher. Run with no arguments for watch mode."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show progress through every module and exercise
    List,
    /// Check an exercise once (default: the current one)
    Check { exercise: Option<String> },
    /// Reveal the next hint for an exercise (default: the current one)
    Hint { exercise: Option<String> },
    /// Take a module's predict-the-output quiz (default: the current module)
    Quiz { module: Option<String> },
    /// Show the reference solution (unlocked once you've solved the exercise)
    Solution {
        exercise: Option<String>,
        /// Show it anyway, marking the exercise as given up
        #[arg(long)]
        give_up: bool,
    },
    /// Restore an exercise to its starting point (your changes are lost)
    Reset { exercise: String },
    /// Put back the original tests at the bottom of an exercise, keeping your code
    RestoreTests { exercise: Option<String> },
    /// Make an exercise, or the first exercise of a module, the current one
    Goto { target: String },
    /// Print a module's refresher notes (default: the current module)
    Notes { module: Option<String> },
    /// Tools for course authors
    #[command(subcommand)]
    Dev(DevCmd),
}

#[derive(Subcommand)]
enum DevCmd {
    /// Check every stub fails, every solution passes and every quiz has one right answer
    Verify {
        /// Only verify ids containing this text
        filter: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{} {e:#}", ui::red("error:"));
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<bool> {
    let mut ctx = Ctx::load()?;
    match cli.command {
        None => watch::run(&mut ctx)?,
        Some(Cmd::List) => screens::list(&ctx),
        Some(Cmd::Check { exercise }) => {
            let ex = pick(&ctx, exercise.as_deref())?;
            work::ensure_exercise(&ctx.course, &ex)?;
            screens::header(&ctx, &ex);
            let outcome = ctx.check_work(&ex, true)?;
            screens::outcome(&ctx, &ex, &outcome);
            return Ok(matches!(outcome, Outcome::Pass { .. }));
        }
        Some(Cmd::Hint { exercise }) => {
            let ex = pick(&ctx, exercise.as_deref())?;
            screens::next_hint(&mut ctx, &ex)?;
        }
        Some(Cmd::Quiz { module }) => {
            let id = match module {
                Some(query) => ctx.course.resolve_module(&query)?.id.clone(),
                None => pick(&ctx, None)?.module,
            };
            quiz::run(&mut ctx, &id, &ui::Input::spawn())?;
        }
        Some(Cmd::Solution { exercise, give_up }) => {
            let ex = pick(&ctx, exercise.as_deref())?;
            if !ctx.progress.is_done(&ex.id) {
                if !give_up {
                    println!(
                        "{} isn't solved yet. Pass --give-up to see the solution anyway.",
                        ex.id
                    );
                    return Ok(false);
                }
                let stats = ctx.progress.stats_mut(&ex.id);
                stats.gave_up = true;
                stats.done = true;
                stats.solved_at = Some(progress::now());
                ctx.progress.save()?;
            }
            screens::solution(&ctx, &ex)?;
        }
        Some(Cmd::Reset { exercise }) => {
            let ex = ctx.course.resolve_exercise(&exercise)?.clone();
            work::reset(&ctx.course, &ex)?;
            println!("Reset {}.", ctx.course.rel(&ex.work).display());
        }
        Some(Cmd::RestoreTests { exercise }) => {
            let ex = pick(&ctx, exercise.as_deref())?;
            work::restore_tests(&ex)?;
            println!(
                "Restored the tests in {}.",
                ctx.course.rel(&ex.work).display()
            );
        }
        Some(Cmd::Goto { target }) => {
            let ex =
                match ctx.course.resolve_exercise(&target) {
                    Ok(ex) => ex.clone(),
                    Err(_) => {
                        let module = ctx.course.resolve_module(&target)?;
                        module.exercises.first().cloned().ok_or_else(|| {
                            anyhow::anyhow!("module {} has no exercises", module.id)
                        })?
                    }
                };
            ctx.progress.current = Some(ex.id.clone());
            ctx.progress.save()?;
            println!("Current exercise: {}. Run `cargo rusty` to start.", ex.id);
        }
        Some(Cmd::Notes { module }) => {
            let id = match module {
                Some(query) => ctx.course.resolve_module(&query)?.id.clone(),
                None => pick(&ctx, None)?.module,
            };
            screens::notes(&ctx, &id)?;
        }
        Some(Cmd::Dev(DevCmd::Verify { filter })) => {
            return verify::run(&ctx, filter.as_deref());
        }
    }
    Ok(true)
}

/// The named exercise, or the current one.
fn pick(ctx: &Ctx, query: Option<&str>) -> Result<course::Exercise> {
    match query {
        Some(q) => Ok(ctx.course.resolve_exercise(q)?.clone()),
        None => ctx
            .current()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("every exercise is solved; name one explicitly")),
    }
}

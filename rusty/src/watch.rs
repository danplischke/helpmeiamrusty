//! Watch mode: show the current exercise, re-check it on every save, move on when it passes.

use std::fs;
use std::time::Duration;

use anyhow::Result;

use crate::course::Exercise;
use crate::ctx::Ctx;
use crate::progress::QuizStats;
use crate::ui::{self, Input, Poll};
use crate::{quiz, screens, work};

const POLL_INTERVAL: Duration = Duration::from_millis(300);

enum Flow {
    Next,
    Step(isize),
    Quit,
}

pub fn run(ctx: &mut Ctx) -> Result<()> {
    let input = Input::spawn();
    loop {
        let Some(ex) = ctx.current().cloned() else {
            finished(ctx);
            return Ok(());
        };
        if !offer_quiz(ctx, &ex, &input)? {
            return Ok(());
        }
        let flow = session(ctx, &ex, &input)?;
        match flow {
            Flow::Next => ctx.advance_from(&ex),
            Flow::Step(offset) => ctx.step_from(&ex, offset),
            Flow::Quit => {}
        }
        ctx.progress.save()?;
        if matches!(flow, Flow::Quit) {
            println!(
                "{}",
                ui::dim("Progress saved. Run `cargo rusty` to pick up here.")
            );
            return Ok(());
        }
    }
}

/// On a module's first exercise, offers its quiz. Returns false if stdin closed.
fn offer_quiz(ctx: &mut Ctx, ex: &Exercise, input: &Input) -> Result<bool> {
    let module = ctx.course.module_of(ex);
    let started = module.exercises.iter().any(|e| {
        ctx.progress
            .get(&e.id)
            .is_some_and(|s| s.opened_at.is_some())
    });
    if module.quiz.is_empty() || started || ctx.progress.quizzes.contains_key(&module.id) {
        return Ok(true);
    }
    let module_id = module.id.clone();
    ui::clear_screen();
    println!(
        "{}\n",
        ui::banner(
            &format!("{} · {}", module.number(), module.title),
            "new module"
        )
    );
    println!(
        "{}",
        ui::wrap(
            &format!(
                "This module opens with a {}-question quiz: read a snippet, predict whether \
                 it compiles and what it prints. Each answer is checked by actually compiling \
                 the code. It finds your blind spots before the exercises do.",
                module.quiz.len()
            ),
            ""
        )
    );
    println!(
        "\nRefresher notes: {}",
        ui::bold(&ctx.course.rel(&module.readme()).display().to_string())
    );
    let Some(answer) = input.ask(&format!("\nTake the quiz now? {} ", ui::dim("[Y/n]"))) else {
        return Ok(false);
    };
    if answer.trim().eq_ignore_ascii_case("n") {
        ctx.progress.quizzes.insert(
            module_id,
            QuizStats {
                skipped: true,
                ..QuizStats::default()
            },
        );
        ctx.progress.save()?;
        return Ok(true);
    }
    quiz::run(ctx, &module_id, input)?;
    Ok(input
        .ask(&format!("\n{}", ui::dim("Enter to start the exercises ")))
        .is_some())
}

fn session(ctx: &mut Ctx, ex: &Exercise, input: &Input) -> Result<Flow> {
    work::ensure_exercise(&ctx.course, ex)?;
    ctx.progress.current = Some(ex.id.clone());
    ctx.progress.touch(&ex.id);
    ctx.progress.save()?;

    let mut seen = fs::read_to_string(&ex.work).unwrap_or_default();
    let mut solved = check(ctx, ex, input, false)?;
    loop {
        match input.poll(POLL_INTERVAL) {
            Poll::Closed => return Ok(Flow::Quit),
            Poll::Timeout => {
                let now = fs::read_to_string(&ex.work).unwrap_or_default();
                if now != seen {
                    seen = now;
                    solved = check(ctx, ex, input, true)?;
                }
            }
            Poll::Line(line) => match line.trim() {
                "" if solved => return Ok(Flow::Next),
                "" | "c" | "check" => solved = check(ctx, ex, input, true)?,
                "n" | "next" if solved => return Ok(Flow::Next),
                "n" | "next" => return Ok(Flow::Step(1)),
                "p" | "prev" => return Ok(Flow::Step(-1)),
                "h" | "hint" => {
                    screens::next_hint(ctx, ex)?;
                }
                "s" | "solution" => {
                    if ctx.progress.is_done(&ex.id) {
                        screens::solution(ctx, ex)?;
                    } else {
                        println!(
                            "{}",
                            ui::dim("Solve it first, or type `give up` to see the reference now.")
                        );
                    }
                }
                "give up" => {
                    let stats = ctx.progress.stats_mut(&ex.id);
                    if !stats.done {
                        stats.gave_up = true;
                        stats.done = true;
                        stats.solved_at = Some(crate::progress::now());
                    }
                    ctx.progress.save()?;
                    screens::solution(ctx, ex)?;
                    println!(
                        "{}",
                        ui::dim("Marked as given up (~ in the list). Enter or n moves on.")
                    );
                    solved = true;
                }
                "l" | "list" => screens::list(ctx),
                "quiz" => {
                    let module_id = ex.module.clone();
                    quiz::run(ctx, &module_id, input)?;
                    input.ask(&format!(
                        "\n{}",
                        ui::dim("Enter to return to the exercise ")
                    ));
                    solved = check(ctx, ex, input, false)?;
                }
                "notes" => screens::notes(ctx, &ex.module)?,
                "reset" => {
                    let prompt = format!(
                        "Overwrite {} with the starting point? {} ",
                        ctx.course.rel(&ex.work).display(),
                        ui::dim("[y/N]")
                    );
                    if input
                        .ask(&prompt)
                        .is_some_and(|a| a.trim().eq_ignore_ascii_case("y"))
                    {
                        work::reset(&ctx.course, ex)?;
                        seen = fs::read_to_string(&ex.work).unwrap_or_default();
                        solved = check(ctx, ex, input, false)?;
                    }
                }
                "restore-tests" => {
                    work::restore_tests(ex)?;
                    seen = fs::read_to_string(&ex.work).unwrap_or_default();
                    solved = check(ctx, ex, input, false)?;
                }
                "q" | "quit" | "exit" => return Ok(Flow::Quit),
                "?" | "help" => screens::help(),
                other => println!(
                    "{}",
                    ui::dim(&format!("Unknown command `{other}`. Type ? for the list."))
                ),
            },
        }
    }
}

/// Clears the screen, runs the check, and prints the result plus a command footer.
fn check(ctx: &mut Ctx, ex: &Exercise, input: &Input, count_failure: bool) -> Result<bool> {
    ui::clear_screen();
    screens::header(ctx, ex);
    println!("{}", ui::dim("  checking…"));
    let outcome = ctx.check_work(ex, count_failure)?;
    input.discard_pending();
    ui::clear_screen();
    screens::header(ctx, ex);
    let passed = screens::outcome(ctx, ex, &outcome);
    let footer = if passed {
        "  Enter next · s compare with the reference solution · ? help · q quit".to_string()
    } else {
        let shown = ctx.progress.get(&ex.id).map_or(0, |s| s.hints);
        format!(
            "  Save to re-check · h hint ({shown}/{}) · ? help · q quit",
            ex.hints.len()
        )
    };
    println!("{}", ui::dim(&footer));
    Ok(passed)
}

fn finished(ctx: &Ctx) {
    ui::clear_screen();
    screens::list(ctx);
    println!(
        "\n{}",
        ui::green("Every exercise is solved. More modules are on the way: `git pull` for updates.")
    );
}

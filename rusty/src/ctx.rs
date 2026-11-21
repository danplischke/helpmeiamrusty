//! Shared state for every command: the course, the learner's progress, and the cargo to use.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::check::{Checker, Outcome, Package};
use crate::course::{Course, Exercise};
use crate::progress::{Progress, now};

pub struct Ctx {
    pub course: Course,
    pub progress: Progress,
    pub cargo: PathBuf,
}

impl Ctx {
    pub fn load() -> Result<Self> {
        let root = Course::find_root()?;
        let course = Course::load(&root)?;
        let progress = Progress::load(course.work_dir().join("progress.toml"))?;
        // `cargo rusty` runs us through Cargo, which tells us which cargo binary it is.
        let cargo = std::env::var_os("CARGO").map_or_else(|| PathBuf::from("cargo"), PathBuf::from);
        Ok(Self {
            course,
            progress,
            cargo,
        })
    }

    pub fn clippy_conf_root(&self) -> PathBuf {
        self.course.scratch_dir().join("clippy")
    }

    /// The exercise to work on: the one last opened, else the first unsolved one.
    pub fn current(&self) -> Option<&Exercise> {
        if let Some(id) = &self.progress.current
            && let Some(ex) = self.course.exercise(id)
        {
            return Some(ex);
        }
        self.first_unsolved()
    }

    pub fn first_unsolved(&self) -> Option<&Exercise> {
        self.course
            .exercises()
            .find(|e| !self.progress.is_done(&e.id))
    }

    /// Moves on to the next unsolved exercise after `ex`, wrapping around to earlier ones.
    pub fn advance_from(&mut self, ex: &Exercise) {
        let all: Vec<&Exercise> = self.course.exercises().collect();
        let start = all.iter().position(|e| e.id == ex.id).map_or(0, |i| i + 1);
        let next = all[start..]
            .iter()
            .chain(&all[..start])
            .find(|e| !self.progress.is_done(&e.id))
            .map(|e| e.id.clone());
        self.progress.current = next;
    }

    /// Moves `offset` exercises forward or back in course order, solved or not.
    pub fn step_from(&mut self, ex: &Exercise, offset: isize) {
        let all: Vec<&Exercise> = self.course.exercises().collect();
        let here = all.iter().position(|e| e.id == ex.id).unwrap_or(0);
        let target = here
            .saturating_add_signed(offset)
            .min(all.len().saturating_sub(1));
        self.progress.current = all.get(target).map(|e| e.id.clone());
    }

    /// Checks the learner's working copy and records the result.
    /// `count_failure` is false for the automatic check when an exercise is opened.
    pub fn check_work(&mut self, ex: &Exercise, count_failure: bool) -> Result<Outcome> {
        let pristine = fs::read_to_string(&ex.stub)?;
        let package = Package {
            manifest: self.course.work_dir().join("Cargo.toml"),
            target_dir: self.course.root.join("target"),
        };
        let conf_root = self.clippy_conf_root();
        let checker = Checker {
            cargo: &self.cargo,
            package: &package,
            clippy_conf_root: &conf_root,
        };
        let outcome = checker.check(ex, &ex.bin(), &ex.work, &pristine)?;

        self.progress.touch(&ex.id);
        let stats = self.progress.stats_mut(&ex.id);
        match outcome {
            Outcome::Pass { .. } if !stats.done => {
                stats.done = true;
                stats.solved_at = Some(now());
            }
            Outcome::Fail { .. } if !stats.done && count_failure => stats.failed_checks += 1,
            _ => {}
        }
        self.progress.save()?;
        Ok(outcome)
    }
}

//! Learner progress, stored in `work/progress.toml`.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Gaps between activity longer than this count as a break, not as time spent.
const IDLE_CUTOFF_SECS: u64 = 10 * 60;

const HEADER: &str = "\
# Your progress through the course, written by `cargo rusty`.
# Safe to commit (so it follows you between machines). Delete it to start over.

";

#[derive(Serialize, Deserialize, Default)]
pub struct Progress {
    #[serde(skip)]
    path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    #[serde(default)]
    pub exercises: BTreeMap<String, ExerciseStats>,
    #[serde(default)]
    pub quizzes: BTreeMap<String, QuizStats>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct ExerciseStats {
    pub done: bool,
    pub gave_up: bool,
    /// Failed checks before the exercise was solved.
    pub failed_checks: u32,
    /// Number of hints revealed.
    pub hints: u32,
    /// Time spent working on it, excluding breaks.
    pub active_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opened_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solved_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<u64>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct QuizStats {
    pub correct: u32,
    pub total: u32,
    pub taken_at: u64,
    /// 1-based numbers of the questions answered wrong.
    pub missed: Vec<u32>,
    pub skipped: bool,
}

impl Progress {
    pub fn load(path: PathBuf) -> Result<Self> {
        let mut progress: Progress = match fs::read_to_string(&path) {
            Ok(text) => {
                toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Progress::default(),
            Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
        };
        progress.path = path;
        Ok(progress)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let body = toml::to_string_pretty(self)?;
        fs::write(&self.path, format!("{HEADER}{body}"))
            .with_context(|| format!("writing {}", self.path.display()))
    }

    pub fn get(&self, id: &str) -> Option<&ExerciseStats> {
        self.exercises.get(id)
    }

    pub fn is_done(&self, id: &str) -> bool {
        self.get(id).is_some_and(|s| s.done)
    }

    pub fn stats_mut(&mut self, id: &str) -> &mut ExerciseStats {
        self.exercises.entry(id.to_string()).or_default()
    }

    /// Records activity on an exercise and accumulates active time.
    pub fn touch(&mut self, id: &str) {
        let now = now();
        let stats = self.stats_mut(id);
        stats.opened_at.get_or_insert(now);
        if let Some(last) = stats.last_activity
            && !stats.done
        {
            let gap = now.saturating_sub(last);
            if gap < IDLE_CUTOFF_SECS {
                stats.active_secs += gap;
            }
        }
        stats.last_activity = Some(now);
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub fn fmt_duration(secs: u64) -> String {
    match secs {
        0..60 => format!("{secs}s"),
        60..3600 => format!("{}m {:02}s", secs / 60, secs % 60),
        _ => format!("{}h {:02}m", secs / 3600, secs % 3600 / 60),
    }
}

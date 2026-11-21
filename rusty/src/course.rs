//! Course content: modules, exercises and quizzes, loaded from TOML files.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;

pub const COURSE_FILE: &str = "course.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CourseFile {
    title: String,
    modules: Vec<String>,
    #[serde(default)]
    dependencies: toml::Table,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleFile {
    title: String,
    #[serde(default, rename = "exercise")]
    exercises: Vec<ExerciseFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExerciseFile {
    name: String,
    kind: Kind,
    #[serde(default)]
    constraints: Vec<Constraint>,
    #[serde(default)]
    hints: Vec<String>,
    fails_at: Option<Stage>,
    #[serde(default = "default_true")]
    run_main: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QuizFile {
    #[serde(default, rename = "question")]
    questions: Vec<Question>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub prompt: String,
    pub code: String,
    pub choices: Vec<String>,
    pub explain: String,
    pub drill: Option<String>,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Fix,
    Implement,
    Constrain,
    Refactor,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Fix => "fix",
            Kind::Implement => "implement",
            Kind::Constrain => "constrain",
            Kind::Refactor => "refactor",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Kind::Fix => "make it compile without changing what it does",
            Kind::Implement => "replace the todo!()s so the tests pass",
            Kind::Constrain => "make it pass while obeying the constraints",
            Kind::Refactor => "turn working-but-clunky code into idiomatic Rust",
        }
    }
}

/// The stages of a check, in the order they run.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Rules,
    Compile,
    Test,
    Clippy,
    Run,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Stage::Rules => "rules",
            Stage::Compile => "compile",
            Stage::Test => "test",
            Stage::Clippy => "clippy",
            Stage::Run => "run",
        })
    }
}

/// A rule an exercise must be solved under, enforced through clippy/rustc lints.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Constraint {
    NoClone,
    NoUnwrap,
    NoIndexing,
    NoUnsafe,
    Pedantic,
}

impl Constraint {
    pub fn label(self) -> &'static str {
        match self {
            Constraint::NoClone => "no .clone()",
            Constraint::NoUnwrap => "no unwrap/expect",
            Constraint::NoIndexing => "no [indexing]",
            Constraint::NoUnsafe => "no unsafe",
            Constraint::Pedantic => "clippy::pedantic",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Constraint::NoClone => "no-clone",
            Constraint::NoUnwrap => "no-unwrap",
            Constraint::NoIndexing => "no-indexing",
            Constraint::NoUnsafe => "no-unsafe",
            Constraint::Pedantic => "pedantic",
        }
    }

    /// Lint flags handed to clippy-driver.
    pub fn lint_args(self) -> &'static [&'static str] {
        match self {
            Constraint::NoClone => &["-D", "clippy::disallowed_methods"],
            Constraint::NoUnwrap => &["-D", "clippy::unwrap_used", "-D", "clippy::expect_used"],
            Constraint::NoIndexing => &["-D", "clippy::indexing_slicing"],
            Constraint::NoUnsafe => &["-F", "unsafe_code"],
            Constraint::Pedantic => &["-D", "clippy::pedantic"],
        }
    }

    /// `(path, reason)` entries for `disallowed-methods` in a generated clippy.toml.
    pub fn disallowed_methods(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Constraint::NoClone => &[(
                "std::clone::Clone::clone",
                "constraint: solve this one without .clone()",
            )],
            _ => &[],
        }
    }

    /// Lints the learner may not silence with `#[allow]` / `#[expect]`.
    pub fn guarded_lints(self) -> &'static [&'static str] {
        match self {
            Constraint::NoClone => &["disallowed_methods"],
            Constraint::NoUnwrap => &["unwrap_used", "expect_used"],
            Constraint::NoIndexing => &["indexing_slicing"],
            Constraint::NoUnsafe => &["unsafe_code"],
            Constraint::Pedantic => &["clippy::pedantic"],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Exercise {
    /// `"<module>/<name>"`, e.g. `"01-ownership/04_split_borrows"`.
    pub id: String,
    pub module: String,
    pub name: String,
    pub kind: Kind,
    pub constraints: Vec<Constraint>,
    pub hints: Vec<String>,
    /// The stage a pristine stub is expected to fail at (checked by `dev verify`).
    pub fails_at: Option<Stage>,
    pub run_main: bool,
    /// The pristine starting point shipped with the course.
    pub stub: PathBuf,
    pub solution: PathBuf,
    /// The learner's working copy.
    pub work: PathBuf,
}

impl Exercise {
    /// Name of the Cargo bin target that builds this exercise.
    pub fn bin(&self) -> String {
        format!("{}__{}", self.module, self.name)
    }
}

pub struct Module {
    pub id: String,
    pub title: String,
    pub dir: PathBuf,
    pub exercises: Vec<Exercise>,
    pub quiz: Vec<Question>,
}

impl Module {
    pub fn number(&self) -> &str {
        self.id.split('-').next().unwrap_or(&self.id)
    }

    pub fn readme(&self) -> PathBuf {
        self.dir.join("README.md")
    }
}

pub struct Course {
    pub root: PathBuf,
    pub title: String,
    pub modules: Vec<Module>,
    pub dependencies: toml::Table,
}

impl Course {
    /// Finds the course root: the nearest ancestor of the working directory with a `course.toml`.
    pub fn find_root() -> Result<PathBuf> {
        let cwd = std::env::current_dir()?;
        if let Some(dir) = cwd.ancestors().find(|d| d.join(COURSE_FILE).is_file()) {
            return Ok(dir.to_path_buf());
        }
        let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        if fallback.join(COURSE_FILE).is_file() {
            return Ok(fallback);
        }
        bail!("couldn't find {COURSE_FILE}; run this from inside the helpmeiamrusty repository")
    }

    pub fn load(root: &Path) -> Result<Self> {
        let file: CourseFile = read_toml(&root.join(COURSE_FILE))?;
        let mut modules = Vec::new();
        for id in &file.modules {
            let dir = root.join("modules").join(id);
            let module_file: ModuleFile = read_toml(&dir.join("module.toml"))?;
            let quiz_path = dir.join("quiz.toml");
            let quiz = if quiz_path.is_file() {
                read_toml::<QuizFile>(&quiz_path)?.questions
            } else {
                Vec::new()
            };
            let exercises = module_file
                .exercises
                .into_iter()
                .map(|e| {
                    let file = format!("{}.rs", e.name);
                    Exercise {
                        id: format!("{id}/{}", e.name),
                        module: id.clone(),
                        stub: dir.join("exercises").join(&file),
                        solution: dir.join("solutions").join(&file),
                        work: root.join("work").join(id).join(&file),
                        name: e.name,
                        kind: e.kind,
                        constraints: e.constraints,
                        hints: e.hints,
                        fails_at: e.fails_at,
                        run_main: e.run_main,
                    }
                })
                .collect();
            modules.push(Module {
                id: id.clone(),
                title: module_file.title,
                dir,
                exercises,
                quiz,
            });
        }
        Ok(Self {
            root: root.to_path_buf(),
            title: file.title,
            modules,
            dependencies: file.dependencies,
        })
    }

    pub fn exercises(&self) -> impl Iterator<Item = &Exercise> {
        self.modules.iter().flat_map(|m| &m.exercises)
    }

    pub fn exercise(&self, id: &str) -> Option<&Exercise> {
        self.exercises().find(|e| e.id == id)
    }

    pub fn module(&self, id: &str) -> Option<&Module> {
        self.modules.iter().find(|m| m.id == id)
    }

    pub fn module_of(&self, ex: &Exercise) -> &Module {
        self.module(&ex.module)
            .expect("every exercise belongs to a loaded module")
    }

    /// Accepts a full id (`01-ownership/04_split_borrows`), a name (`04_split_borrows`)
    /// or any unique fragment (`split`).
    pub fn resolve_exercise(&self, query: &str) -> Result<&Exercise> {
        if let Some(ex) = self.exercise(query) {
            return Ok(ex);
        }
        let by_name: Vec<&Exercise> = self.exercises().filter(|e| e.name == query).collect();
        if let [one] = by_name[..] {
            return Ok(one);
        }
        let fuzzy: Vec<&Exercise> = self.exercises().filter(|e| e.id.contains(query)).collect();
        match fuzzy[..] {
            [one] => Ok(one),
            [] => bail!("no exercise matches `{query}` (see `cargo rusty list`)"),
            _ => bail!(
                "`{query}` matches several exercises: {}",
                fuzzy
                    .iter()
                    .map(|e| e.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    /// Accepts a module id (`01-ownership`), its number (`01`, `1`) or a unique fragment.
    pub fn resolve_module(&self, query: &str) -> Result<&Module> {
        let wanted_number = query.parse::<u32>().ok();
        if let Some(m) = self
            .modules
            .iter()
            .find(|m| m.id == query || wanted_number.is_some_and(|n| m.number().parse() == Ok(n)))
        {
            return Ok(m);
        }
        let fuzzy: Vec<&Module> = self
            .modules
            .iter()
            .filter(|m| m.id.contains(query))
            .collect();
        match fuzzy[..] {
            [one] => Ok(one),
            [] => bail!("no module matches `{query}` (see `cargo rusty list`)"),
            _ => bail!("`{query}` matches several modules"),
        }
    }

    pub fn work_dir(&self) -> PathBuf {
        self.root.join("work")
    }

    /// Scratch space under `target/` for generated files.
    pub fn scratch_dir(&self) -> PathBuf {
        self.root.join("target").join("rusty")
    }

    /// A path relative to the course root, for display.
    pub fn rel<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}

fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

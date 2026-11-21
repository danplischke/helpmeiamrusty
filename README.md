# helpmeiamrusty

An interactive Rust refresher for people who **used to write Rust** and want
to be fluent again. Quizzes show what you've forgotten, the compiler does the
teaching, and clippy holds you to idiomatic code.

```text
━━ 01 · Ownership & borrowing ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 3/9 ━━
▶ 04_split_borrows  fix: make it compile without changing what it does
  rules  no .clone() · no [indexing] · no unsafe
  edit   work/01-ownership/04_split_borrows.rs
────────────────────────────────────────────────────────────────────────
error[E0499]: cannot borrow `*self` as mutable more than once at a time
...
✗ Doesn't compile yet.
  Save to re-check · h hint (0/3) · ? help · q quit
```

## Quick start

You need [rustup](https://rustup.rs). The course pins the toolchain it was
verified on (Rust 1.97.0, in `rust-toolchain.toml`), and rustup installs
it, with clippy and rustfmt, the first time you run a command here.

```bash
git clone https://github.com/danplischke/helpmeiamrusty
```

```bash
cd helpmeiamrusty
```

```bash
cargo rusty
```

Always run `cargo rusty` from the repository root. Open the file it names
under `work/` in your editor and save to re-check. It moves on by itself
once you pass.

## How it works

**Each module opens with a quiz.** You read a short snippet and predict
whether it compiles and what it prints. The runner compiles and runs every
snippet for real, so the answer always matches your toolchain. Wrong answers
point at the exercise that drills that rule.

**Then exercises, of four kinds:**

| Kind | You… |
|---|---|
| fix | make broken code compile without changing what it does |
| implement | replace `todo!()`s until the tests pass |
| constrain | solve under rules like *no `.clone()`*, *no `unwrap`*, *no indexing* |
| refactor | turn working but clunky code into idiomatic Rust |

**Every save runs the same pipeline:** rules → compile → tests → clippy →
run `main`. Rules means the tests at the bottom of each file are untouched
and no constraint lint is silenced with `#[allow]`. Clippy's default lints
count as errors here, and each exercise's constraints are enforced as clippy
lints.

**Your work stays separate from the course.** The runner copies each
exercise into `work/` the first time you open it, and you only edit those
copies. Course content in `modules/` never changes under you, so `git pull`
for new modules never conflicts with your work. `work/progress.toml` records
time spent, failed checks and hints used. Commit `work/` if you want your
progress on another machine.

## Commands

| Command | What it does |
|---|---|
| `cargo rusty` | Watch mode: the current exercise, re-checked on every save |
| `cargo rusty list` | Progress through every module |
| `cargo rusty check [exercise]` | Check once |
| `cargo rusty hint [exercise]` | Next hint (three per exercise, from a nudge to almost the answer) |
| `cargo rusty quiz [module]` | Take or retake a module's quiz |
| `cargo rusty solution [exercise]` | Reference solution with notes on why it's idiomatic. Unlocks once solved, or with `--give-up` |
| `cargo rusty goto <exercise or module>` | Jump anywhere, e.g. `cargo rusty goto 01` |
| `cargo rusty reset <exercise>` | Back to the starting point |
| `cargo rusty restore-tests [exercise]` | Put the original tests back, keeping your code |
| `cargo rusty notes [module]` | Print a module's refresher notes |

Exercises can be named by full id (`01-ownership/04_split_borrows`), by name
(`04_split_borrows`) or by any unique fragment (`split`). In watch mode, the
same commands are single keys: `h`, `s`, `n`, `p`, `l`, `quiz`, `notes`,
`reset`, `?`.

## Claude Code as your tutor

[`CLAUDE.md`](CLAUDE.md) puts Claude Code in tutor mode for this repository.
It won't write your exercises unless you ask it to. It explains compiler
errors, climbs a hint ladder, and suggests small standalone examples. Two
skills come with it:

- `/critique [exercise]`: once an exercise passes, a review for idiom against
  the reference solution. You get the one change worth making, not a rewrite.
- `/drill [topic]`: fresh "does it compile?" questions on a weak spot, each
  verified with the compiler before it's asked.

## What's here so far

| Module | Exercises | Quiz |
|---|---|---|
| 00 · Warm-up & syntax speedrun | 7 | 10 questions |
| 01 · Ownership & borrowing | 9 | 10 questions |
| 02 · Modeling with types | 10 | 10 questions |
| 03 · Traits & generics | 10 | 10 questions |
| 04 · Lifetimes in practice | 10 | 10 questions |
| 05 · Error handling | 10 | 10 questions |
| 06 · Iterators & collections | 10 | 10 questions |

The full curriculum is laid out in [PROPOSAL.md](PROPOSAL.md): 14 modules,
six projects, a placement test, a "what's new since you left" track and
spaced review.

## Working on the course

```bash
cargo rusty dev verify
```

This checks that every stub fails at the stage it claims, every reference
solution passes the stub's exact tests and rules, every file is
rustfmt-clean, and every quiz question has exactly one right answer. CI runs
it on every push.

[AUTHORING.md](AUTHORING.md) has the conventions. A module is a folder in
`modules/` listed in `course.toml`:

```text
modules/01-ownership/
├── README.md        # one-page refresher
├── module.toml      # exercises in order: kind, rules, hints, expected failure stage
├── quiz.toml        # snippets and choices; the runner works out the answers
├── exercises/       # starting points; tests at the bottom under #[cfg(test)]
└── solutions/       # reference solutions with the same tests, plus notes
```

# Changelog

## [0.1.0] - 2026-11-21

The first release: the `cargo rusty` runner and the first two modules.

### Course

- **00 · Warm-up & syntax speedrun**: 7 exercises (bindings, expressions,
  strings, structs and derives, enums and patterns, collections, modules),
  plus a 10-question quiz.
- **01 · Ownership & borrowing**: 9 exercises (moves, shared XOR mutable,
  returning references, split borrows, `mem::take`/`replace`, slice
  parameters, the entry API, mutating while iterating, and an inventory
  checkpoint), plus a 10-question quiz.

### Runner

- **Watch mode** (`cargo rusty`) re-checks the current exercise on every
  save. A check runs rules → compile → tests → clippy → `main`.
- **Per-exercise constraints** are enforced with clippy: no `.clone()`, no
  `unwrap`, no indexing, no `unsafe`, pedantic. Editing the tests or
  silencing a constraint lint is rejected.
- **Hints and solutions:** three hints per exercise, and reference solutions
  with notes on why they're idiomatic, unlocked once you've solved the
  exercise.
- **Real quizzes:** every snippet is compiled and run, so the right answer
  always matches your toolchain.
- **Progress tracking** in `work/progress.toml`: time spent, failed checks,
  hints used and quiz scores.
- **Course verification** with `cargo rusty dev verify`, run by CI: every
  stub fails where it should, every solution passes, and every quiz question
  has exactly one answer.

### Tutor mode

- `CLAUDE.md` makes Claude Code a tutor rather than a solver.
- `/critique` reviews a solved exercise; `/drill` asks compiler-verified
  practice questions.

### Getting started

You need Rust 1.88 or newer through rustup. Clone the repository and run
`cargo rusty` from its root.

### Coming next

Modules 02–06: modeling with types, traits and generics, lifetimes, error
handling, and iterators and collections.

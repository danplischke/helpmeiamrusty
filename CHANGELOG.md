# Changelog

## [0.2.0] - 2026-11-21

Five more modules: the course now covers the whole core of the language.
That's 66 exercises and 70 quiz questions.

### Course

- **02 · Modeling with types**: exhaustive matching, slice patterns, nested
  patterns and `@` bindings, edition 2024 binding modes, let-else and let
  chains, unit newtypes, parse-don't-validate, illegal states made
  unrepresentable, typestate, and an order state-machine checkpoint.
- **03 · Traits & generics**: bounds that match the body, `impl Trait`
  vs `dyn Trait`, default methods and associated consts, associated types vs
  generic parameters, the orphan rule, dyn compatibility and trait upcasting,
  conversion traits, `Eq`/`Hash`/`Ord` consistency and `Borrow`, operators on
  references, and a typed `Money` checkpoint.
- **04 · Lifetimes in practice**: elision rules, whose data a result
  borrows, structs that borrow, tokens that outlive their lexer, two
  lifetimes and invariance, `Box<dyn Fn + 'a>` and `'static`, higher-ranked
  bounds, edition 2024 `use<..>` precise capturing, scoped threads, and a
  zero-copy log-reader checkpoint.
- **05 · Error handling**: `Option` ↔ `Result`, `?` and `From`,
  hand-written error enums, thiserror, anyhow context, collecting `Result`s,
  `Box<dyn Error + Send + Sync>`, a no-unwrap drill, and an INI config parser
  checkpoint.
- **06 · Iterators & collections**: adaptors, laziness, windows and chunks,
  loops rewritten as chains, sorting and searching, custom iterators,
  `IntoIterator` for references, collecting into anything,
  `VecDeque`/`BinaryHeap`/`BTreeMap::range`, and a log-analyzer checkpoint.

Each module has 10 exercises and a 10-question quiz.

### Runner and tooling

- **Pinned toolchain.** The course is pinned to Rust 1.97.0, the version
  every exercise and quiz answer is verified on. rustup installs it
  automatically.
- **Error exits in quizzes.** A quiz can now ask about a `main` that
  returns `Err`: it matches "exits with an error", not "panics".
- **Scoped verification.** `cargo rusty dev verify <filter>` builds only the
  selected exercises, so work in progress elsewhere can't break it.
- **Authoring guide.** `AUTHORING.md` documents the conventions every
  module follows.

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

# helpmeiamrusty — an interactive Rust refresher

> A terminal-first course for someone who **already knew Rust** and wants
> fluency back: fast where you remember, deep where you've gone rusty.

## 1. The idea in three principles

1. **Retrieval before reading.** Every module opens with a short *"does this
   compile? what does it print?"* quiz. It shows what you've forgotten before
   you read any explanation, so you only read what you need.
2. **The compiler teaches.** Exercises are broken code, `todo!()`s backed by
   tests, and *constraint challenges* that clippy enforces (e.g. "no
   `.clone()`", "no `unwrap`", "no indexing"). You learn by getting errors and
   fixing them, not by reading.
3. **Fluency = speed + recall.** The runner records how long each exercise takes
   and how many hints you used. Exercises you struggled with come back days
   later, reset to their starting state (spaced repetition).

## 2. What a session feels like

```text
$ cargo rusty
 Module 03 · Traits & generics                 ▓▓▓▓▓▓░░░░  6/10
 ▶ 03_07_blanket_impl.rs   [implement]   constraint: no Box<dyn _>
 ✗ 2 tests failing. Edit the file and it re-checks on save.
 [h] hint 1/3   [s] solution (locked)   [q] quiz   [n] skip
```

```text
$ cargo rusty quiz ownership
 Q3/8 · Does this compile?

     let mut v = vec![1, 2, 3];
     let first = &v[0];
     v.push(4);
     println!("{first}");

 [y/n] > y
 ✗ No. error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
   Why: `push` may reallocate and leave `first` dangling.
   Drill it: exercise 01_04_split_borrows
```

Quiz answers come from actually compiling and running each snippet, so they
always match the toolchain you're using. Nothing is hardcoded.

## 3. Exercise types

| Kind          | You…                                                   | Checked by                                    |
|---------------|--------------------------------------------------------|-----------------------------------------------|
| **Predict**   | answer "compiles?" / "what prints?"                    | runner compiles and runs the snippet          |
| **Fix**       | make broken code compile without changing its intent   | `rustc` + tests                               |
| **Implement** | fill in `todo!()`s                                     | unit tests                                    |
| **Constrain** | solve under a rule: no `clone`, `unwrap`, indexing, `unsafe`… | clippy with a per-exercise config      |
| **Refactor**  | rewrite working but clunky code                        | tests + `clippy::pedantic` clean              |
| **Milestone** | build one stage of a capstone project                  | integration tests per stage                   |

## 4. Curriculum

| #  | Module                              | What it covers                                                                                                                 |
|----|-------------------------------------|--------------------------------------------------------------------------------------------------------------------------------|
| 00 | Warm-up & tooling                   | Edition 2024, cargo / clippy / rustfmt / rust-analyzer, a syntax speedrun, the **placement test**                              |
| 01 | Ownership & borrowing               | moves vs copies, reborrows, NLL, two-phase borrows, `mem::{take, replace, swap}`, split borrows, `split_at_mut`                |
| 02 | Modeling with types                 | enums and exhaustive `match`, `let-else`, let chains, `@` bindings, slice patterns, newtypes, typestate, `#[non_exhaustive]`    |
| 03 | Traits & generics                   | static vs `dyn` dispatch, associated types, blanket impls, orphan rule, `From/TryFrom/AsRef/Borrow/Deref`, operator overloading, dyn compatibility, trait upcasting |
| 04 | Lifetimes in practice               | elision rules, structs that hold references, `'static` myths, HRTBs (`for<'a>`), variance (the basics), `impl Trait + use<'a>` |
| 05 | Error handling                      | `?` + `From`, custom error enums, `thiserror` vs `anyhow`, adding context, panic vs `Result`, `#[must_use]`                    |
| 06 | Iterators & collections             | adaptor fluency, implementing `Iterator`/`IntoIterator` (for `&T`, `&mut T`), collecting into `Result`/`HashMap`, entry API, `windows/chunks/retain`, `from_fn/successors` |
| 07 | Closures                            | `Fn`/`FnMut`/`FnOnce`, `move`, returning and boxing closures, async closures                                                    |
| 08 | Smart pointers & interior mutability| `Box`, `Rc`/`Weak`, `Cell`/`RefCell`, `Cow`, `OnceLock`/`LazyLock`, drop order; trees and graphs without pain                    |
| 09 | Concurrency                         | threads, `thread::scope`, channels, `Send`/`Sync` (and why `Rc` is neither), `Arc<Mutex<_>>`, `RwLock`, atomics and memory ordering basics, rayon |
| 10 | Async                               | how futures and polling work, tokio, `select!`, cancellation safety, `JoinSet`, streams, timeouts, `async fn` in traits, just enough `Pin` |
| 11 | Macros                              | `macro_rules!` (repetition, hygiene, tt-munching), when a proc macro is worth it, plus one small derive                        |
| 12 | Unsafe & FFI                        | raw pointers, writing `// SAFETY:` contracts, `MaybeUninit`, checking with **Miri**, calling C                                 |
| 13 | Crafting crates                     | modules and visibility, workspaces, features, docs and doctests, property tests, benchmarks, the API guidelines                 |

Each module follows the same loop:
**quiz (10 min) → 1-page refresher README (5 min) → 6–12 exercises → checkpoint.**

### "What's new since you left": a track that runs alongside the modules

`cargo rusty whatsnew --since 1.6x` builds a mini-module of quizzes and
exercises covering only the features stabilized after the version you last
used. For example: `let-else` and GATs (1.65), `OnceLock` (1.70), `async fn`
and `impl Trait` in traits (1.75), C-string literals (1.77),
`#[diagnostic::on_unimplemented]` (1.78), inline `const` (1.79), `LazyLock`
(1.80), `#[expect]` (1.81), precise capturing `use<..>` (1.82), Edition 2024
and async closures (1.85), trait upcasting (1.86), let chains (1.88), and the
newer releases. Each item gets checked against the official release notes
when it's written.

### Placement test

About 25 quiz items spread across all modules (~15 minutes). The result is a
suggested path that marks each module as **skip / skim / do**, so you spend
time only where you've actually gone rusty.

## 5. Capstone projects

Each capstone is built in stages, and every stage has its own integration
tests (`cargo test --test milestone_3`). Each one comes after the modules it
draws on.

| Project   | After   | What you build                                                                                                     |
|-----------|---------|--------------------------------------------------------------------------------------------------------------------|
| **logq**  | 05–06   | A log-query CLI: zero-copy `&str` parsing, filters, aggregates, clap, real error messages. Stretch goal: parallelize with rayon. |
| **calc**  | 02,04,08| Tokenizer → parser → interpreter for a tiny language: enums, `Box`, a lexer that borrows its input, errors with source spans. |
| **kvs**   | 03–08   | A persistent key-value store: log-structured storage, compaction, a trait-based engine, serde.                        |
| **pool**  | 09      | A thread pool with graceful shutdown: `Arc`/`Mutex`/`Condvar`, channels, `Drop`, surviving a panic in a worker.        |
| **chatd** | 10      | An async TCP chat server: tokio, broadcast channels, `select!`, cancellation, backpressure, graceful shutdown.        |
| **arena** | 12      | An arena allocator plus an intrusive linked list in `unsafe`. Done when Miri finds no errors.                         |

## 6. Repository layout

```text
helpmeiamrusty/
├── Cargo.toml                 # workspace, edition 2024
├── rust-toolchain.toml        # pinned stable toolchain (+ clippy, rustfmt)
├── .cargo/config.toml         # `cargo rusty` alias
├── rusty/                     # the runner CLI
├── modules/
│   └── 01-ownership/
│       ├── README.md          # 1-page refresher: mental model + gotchas
│       ├── module.toml        # order, kinds, hints, constraints per exercise
│       ├── quiz.toml          # predict-the-output snippets + explanations
│       ├── exercises/         # one bin target per exercise
│       └── solutions/         # reference solutions + notes on why they're idiomatic
├── projects/                  # capstones; tests/milestone_N.rs per stage
├── progress/                  # your progress (committed, so it follows you between machines)
├── CLAUDE.md                  # tutor rules for Claude Code (see §8)
└── .github/workflows/ci.yml   # checks every solution passes and every stub fails
```

## 7. The runner: `cargo rusty`

| Command                  | Does                                                                                                      |
|--------------------------|-----------------------------------------------------------------------------------------------------------|
| `cargo rusty`            | Watch mode: shows the current exercise, re-checks it on save, moves to the next one when it passes         |
| `hint`                   | Three hint levels: a nudge → the concept plus a docs link → nearly the solution                           |
| `quiz [module]`          | Predict-the-output quiz, verified by really compiling and running each snippet                            |
| `solution`               | Unlocks once you pass (or with `--give-up`); shows the reference solution, idiom notes and a diff against your code |
| `review`                 | Spaced repetition: brings back exercises you were slow on or needed hints for, reset to their starting state |
| `placement`, `whatsnew`  | See §4                                                                                                     |
| `status`, `reset <ex>`   | Progress map; restore an exercise to its starting state                                                    |

**How it works, already tested on the toolchain in this environment (Rust 1.97):**

- **Each exercise is its own `[[bin]]` target.** `cargo test --bin <ex>`
  compiles only that exercise, so an unfinished, broken exercise never blocks
  the others.
- **Constraints are enforced by clippy.** The runner passes lint flags
  (`-D clippy::unwrap_used`, `-D clippy::indexing_slicing`, …) and points
  `CLIPPY_CONF_DIR` at a per-exercise `clippy.toml` that uses
  `disallowed-methods`. In the prototype, calling `.clone()` failed with
  *"use of a disallowed method … constraint: solve without clone"*.
- The runner stays small (about 1–1.5k lines): `clap`, `notify`,
  `serde`/`toml`, and plain ANSI output.

**Alternative considered:** rustlings v6 can host third-party exercise sets.
That gives a polished watch mode for free, but you'd lose quizzes, constraint
challenges, spaced review and placement, which are the parts that matter most
for a *refresher*. **Recommendation: a custom runner.**

## 8. Claude Code as a tutor

You're already working with Claude Code, so the repo ships a `CLAUDE.md`
that switches it into tutor mode:

- It never writes solutions into exercise files unless you explicitly say
  "show me".
- When you ask for help, it asks what the compiler said, explains the error
  code, and walks you through the hints one level at a time.
- `/review`: once you pass an exercise, it critiques your code for idiom,
  compares it with the reference solution, and suggests the one change worth
  making.
- `/drill <topic>`: it writes fresh "does it compile?" questions on a topic
  you're weak in.

## 9. Build plan

| Phase                     | Scope                                                                                       | Outcome                               |
|---------------------------|---------------------------------------------------------------------------------------------|---------------------------------------|
| **1 · Vertical slice**    | Workspace, runner MVP (watch, verify, hint, status), modules 00 and 01 complete with quizzes, `CLAUDE.md` | You try it; we adjust the format |
| **2 · Core fluency**      | Modules 02–06, capstones *logq* and *calc*, solution unlock and idiom notes, CI             | The "I can write Rust again" core     |
| **3 · Depth**             | Modules 07–10, capstones *kvs*, *pool* and *chatd*, spaced review, placement, `whatsnew`     | Concurrency and async fluency          |
| **4 · Advanced**          | Modules 11–13, capstone *arena*, Miri integration                                           | The full course                       |

**Size:** about 120 exercises, about 100 quiz items and 6 capstones, or
roughly 40–60 hours end to end. The placement test usually cuts that a lot.

## 10. Decisions for you

1. **When did you last write Rust seriously** (roughly which year or version)?
   This sets what the "what's new" track and the placement test focus on.
2. **What do you want Rust for now?** Backend/async services, systems/CLI,
   embedded, data/performance, wasm, interviews? This changes how much weight
   each module gets and which capstones we build.
3. **Time budget?** Short daily drills (30 min) or weekend projects? This
   decides how big each module is.
4. **Custom runner (recommended) or exercises hosted on rustlings?**
5. **Ecosystem crates in exercises** (tokio, serde, thiserror, anyhow, rayon,
   clap) or standard library only where possible?

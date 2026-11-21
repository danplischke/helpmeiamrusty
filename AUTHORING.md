# Writing course content

How to write a module for helpmeiamrusty. Modules `00-warmup` and
`01-ownership` are the reference examples: read one stub/solution pair, a
`module.toml`, a `quiz.toml` and a `README.md` before writing your own.

The learner **already knew Rust** and is regaining fluency. Skip "what is a
variable". Go for the things people forget, get subtly wrong, or never
learned properly, and for idioms that separate fluent code from code that
merely compiles.

## A module

```text
modules/NN-topic/
├── README.md        # one-page refresher
├── module.toml      # exercises in order
├── quiz.toml        # 10 predict-the-output questions
├── exercises/       # starting points (what the learner gets)
└── solutions/       # reference solutions
```

A module has **8–10 exercises**. The last one is a **checkpoint** that
combines the module's ideas in one small, realistic type or program.

`cargo rusty dev verify NN-topic` must pass, and it checks:

- every stub **fails** at the stage given by `fails_at`;
- every solution **passes** with the stub's exact tests and rules;
- both files are rustfmt-clean (`rustfmt --edition 2024 <file>` fixes that);
- every quiz question has **exactly one** choice matching what the compiler
  and program actually do;
- a README exists, every exercise has at least two hints, and every quiz
  `drill` names a real exercise.

## Exercises

Every exercise is one self-contained file: a `fn main` plus a
`#[cfg(test)] mod tests` that comes **last** in the file. Each file is built
as its own binary in Rust edition 2024. Exercises may use `std`, plus
`anyhow` and `thiserror` (see `course.toml`).

### Kinds and `fails_at`

| kind | the stub… | usual `fails_at` |
|---|---|---|
| `fix` | doesn't compile; the learner fixes it without changing behaviour | `compile` |
| `implement` | compiles with `todo!()` bodies; tests fail | `test` |
| `constrain` | works but breaks a rule; clippy rejects it | `clippy` |
| `refactor` | works but is clunky; tests or clippy push towards the idiom | `compile`, `test` or `clippy` |

A check runs **rules → compile → tests → clippy → run `main`**. Clippy runs
with `-D clippy::all` plus the exercise's constraints, and `dead_code` is
allowed. `main` must exit successfully within 10 seconds, so keep it a short
demo that uses the functions.

### Constraints

Use `constraints = [...]` in `module.toml`, choosing from:

| constraint | enforced by |
|---|---|
| `no-clone` | `disallowed-methods` on `Clone::clone` |
| `no-unwrap` | `clippy::unwrap_used`, `clippy::expect_used` |
| `no-indexing` | `clippy::indexing_slicing` |
| `no-unsafe` | `-F unsafe_code` |
| `pedantic` | `clippy::pedantic` |

Constraints apply to non-test code only. Tests may `unwrap`, index and clone.

### The stub

- **Header first**, as `//!` docs: `# Title`, then two to six lines on what
  to do and the one or two facts to have in mind. State the constraints in
  plain words if there are any. No "Exercise 3" numbering in titles.
- **A realistic little domain**, not `foo`/`bar`: an inventory, sessions, a
  log parser, versions, a robot.
- **Doc comments on every function** say exactly what it must do, including
  edge cases. The tests must not hold surprises the docs don't mention.
- **Avoid unused-variable warnings in `todo!()` stubs** by using the
  parameters in the message: `todo!("parse {s:?}")`. For a parameter that
  isn't `Debug` (a closure, a `Formatter`), add
  `let _ = f; // only here to silence "unused variable"`.
- **`fix` stubs must still parse.** Use type, borrow and privacy errors, not
  syntax errors, so rustfmt can format them.

### The tests

- **They're the spec, and the learner can't change them.** The runner
  compares the tests section with the pristine copy.
- **Cover the edge cases**: empty input, ties, Unicode, boundaries.
- **Pin the lesson down where you can.** For example, `01_moves` calls
  functions with `&words[..1]` and string literals, so only a borrowed
  signature compiles. `03_return_refs` drops `needle` early so only the
  right lifetime annotation works, and checks with a pointer that the result
  borrows from the input.
- **No nondeterminism**: no `HashMap` iteration order, timing, addresses or
  thread interleavings.

### The solution

- Same file, with the header's title suffixed `(reference solution)`. The
  header becomes **notes**: three to six bullets on *why* this is the
  idiomatic answer, which alternatives exist, and what to remember.
- **The tests section is byte-for-byte the stub's.** The verifier compares
  them, ignoring whitespace.
- It must pass clippy's default lints and the exercise's constraints.

### Hints

Exactly **three hints** per exercise, climbing a ladder:

1. a nudge: which error to read first, what question to ask;
2. the concept or API to reach for;
3. nearly the answer: the key line or the shape of the code.

**Any claim that clippy or the compiler will say something must be true.**
Check it before writing it: copy the solution into a scratch Cargo package
under `target/rusty/scratch-<module>/`, make the change, and run
`cargo clippy -- -D clippy::all`.

## Quizzes

`quiz.toml` holds 10 `[[question]]`s with `prompt`, `code` (a complete
program with `fn main`), `choices`, `explain` and an optional `drill` (an
exercise id like `"02-types/03_slice_patterns"`).

- The runner compiles each snippet with `rustc --edition 2024`, debug
  assertions on and warnings allowed, then runs it. The right choice is
  whichever matches: the trimmed **stdout**, or one of the special choices
  `"does not compile"`, `"panics"` (exit code 101), `"exits with an error"`
  (any other failure, e.g. `main` returning `Err`), `"runs forever"`,
  `"compiles"`.
- **Print on one line** (use `print!` plus a final `println!`), so choices
  stay readable.
- **Exactly one choice may match**, so don't offer `"compiles"` alongside an
  output.
- **Make the wrong choices plausible.** Each one should be what someone with
  a specific misconception would pick.
- **Mix the question types**: "does this compile?", "what does it print?",
  "what happens?". About half should surprise a returning Rust developer.
- **`explain`** is two to five sentences: the rule, why it exists, and the
  practical upshot. Name error codes where they help (E0502).
- **No nondeterminism.**

## The README

About one page, as in `01-ownership/README.md`: the mental model in a few
lines, a "symptom → fix" or "pattern → tool" table, a short code cheat
sheet, and the gotchas the module drills. It's for re-reading later, so
every line has to earn its place.

## Style

- Plain, direct English. Short sentences. Explain *why*, not just *what*.
- No filler ("simply", "just", "easy"), no exclamation marks, no emoji.
- **Version claims must be right.** Only cite a version when you're sure.
  These are safe:

  | Version | Stabilized |
  |---|---|
  | 1.65 | let-else, GATs |
  | 1.70 | `OnceCell` / `OnceLock` |
  | 1.75 | `async fn` and return-position `impl Trait` in traits |
  | 1.77 | `slice::chunk_by`, C-string literals |
  | 1.78 | `#[diagnostic::on_unimplemented]` |
  | 1.79 | inline `const {}` blocks |
  | 1.80 | `LazyCell` / `LazyLock` |
  | 1.81 | `#[expect]`, `core::error::Error` |
  | 1.82 | `use<..>` precise capturing, `iter::repeat_n`, `Option::is_none_or` |
  | 1.85 | edition 2024, async closures |
  | 1.86 | trait upcasting, `get_disjoint_mut` |
  | 1.87 | `Vec::extract_if`, `is_multiple_of` |
  | 1.88 | let chains (edition 2024) |

  Leave the version out otherwise. The course is pinned to 1.97.0
  (`rust-toolchain.toml`), so a feature that compiles may be newer than you
  think.
- **Toolchain quirks worth knowing:**
  - With `no-clone`, don't `#[derive(Clone)]` on types with non-`Copy`
    fields. Clippy flags the `clone` calls inside the derive's expansion.
  - Clippy 1.97's `manual_checked_ops` rejects `if d == 0 { 0 } else { x / d }`.
    Write `x.checked_div(d).unwrap_or(0)`.

## Workflow

```bash
rustfmt --edition 2024 modules/NN-topic/exercises/*.rs modules/NN-topic/solutions/*.rs
```

```bash
cargo rusty dev verify NN-topic
```

Validate TOML before saving it, for example with
`python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" file.toml`.
A broken `module.toml` or `quiz.toml` stops the whole course from loading.

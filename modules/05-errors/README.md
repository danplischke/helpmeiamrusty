# 05 · Error handling

Rust splits failure in two. `Result` is for things that go wrong in a
correct program: bad input, a missing file, a refused connection. `panic!`
is for bugs, where the program's own assumptions broke. Most error handling
is moving errors upward with `?` and making them say something useful on
the way.

## The model in five lines

1. `Option<T>` is a value or nothing; `Result<T, E>` is a value or the
   reason there isn't one. `Result` is `#[must_use]`: ignoring one is a
   warning, and the failure disappears.
2. `expr?` on `Err(e)` returns `Err(From::from(e))`; on `None` it returns
   `None`. It works only on the kind the function (or closure) returns.
3. An error type is `Debug + Display + std::error::Error`. `source()` points
   at the cause, so errors form a **chain**.
4. `Display` shows **one layer**. Put the cause in your message *or* return
   it from `source()`, never both.
5. Libraries return **typed errors** (an enum, usually via thiserror) so
   callers can match on them. Applications **report** errors: anyhow plus
   context.

## Pattern → tool

| You have | You want | Reach for |
|---|---|---|
| `Option<T>` | `Result<T, E>` | `ok_or(e)`, `ok_or_else(\|\| e)` when `e` is costly, anyhow's `.context(..)` |
| `Result<T, E>` | `Option<T>` | `.ok()` (drops the error) |
| `Option<Result<T, E>>` | `Result<Option<T>, E>` | `.transpose()` |
| `Result<T, E1>` in a function returning `E2` | propagate it | `?` plus `impl From<E1> for E2`, or `.map_err(E2::Variant)?` |
| an error without context | "which line? which file?" | `.map_err(\|source\| E::X { line, source })?`, `.with_context(\|\| ..)?` |
| an iterator of `Result<T, E>` | everything, or the first error | `.collect::<Result<Vec<T>, E>>()`, `.sum::<Result<T, E>>()` |
| an iterator of `Result<T, E>` | the good ones *and* the bad ones | a loop pushing into two Vecs, or `partition(Result::is_ok)` |
| a script or prototype | any error through `?` | `Box<dyn Error + Send + Sync>`, or `anyhow::Result` |
| a check, then `unwrap()` | no panic path | `let Some(x) = .. else { .. }`, `if let`, `unwrap_or`, `map_or` |

## A typed error, by hand and derived

```rust
#[derive(Debug)]
enum LoadError {
    Io(io::Error),
    BadLimit { line: usize, source: ParseIntError },
}
impl fmt::Display for LoadError { /* one write! per variant, cause left out */ }
impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LoadError::Io(e) => Some(e),
            LoadError::BadLimit { source, .. } => Some(source),
        }
    }
}
impl From<io::Error> for LoadError { /* LoadError::Io(e) */ }

// The same with thiserror:
#[derive(Debug, thiserror::Error)]
enum LoadError {
    #[error("could not read the file")]
    Io(#[from] io::Error),                 // From impl + source()
    #[error("line {line}: invalid limit")]
    BadLimit { line: usize, source: ParseIntError }, // a `source` field is the cause
    #[error(transparent)]
    Quota(#[from] QuotaError),             // message and cause are QuotaError's
}
```

`std::error::Error` is also `core::error::Error` (Rust 1.81), so `no_std`
libraries can implement it too.

## anyhow at the application edge

```rust
use anyhow::{Context, Result, bail, ensure};

fn load(path: &Path) -> Result<Config> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    ensure!(!text.trim().is_empty(), "{} is empty", path.display());
    let Some(first) = text.lines().next() else { bail!("no lines") };
    // ...
}

// {}   "reading app.ini"                          (outermost layer only)
// {:#} "reading app.ini: No such file or directory (os error 2)"
// {:?} a multi-line report with "Caused by:"     (what `main` prints)
// err.downcast_ref::<io::Error>()                (the original, under the context)
```

## Gotchas this module drills

- **`#[from]` is all or nothing.** The variant can't hold other fields, and
  two variants can't both be `#[from]` one type (E0119). For context, drop
  `#[from]` and `map_err` at the call site.
- **`#[error("{0}")]` around a `#[from]` error prints it twice** in a chain:
  once as the message, once as `source()`. `#[error(transparent)]` doesn't.
- **`?` in a closure** returns from the closure, not the function around it.
- **`ok_or(make_error())` always calls `make_error()`.** Use `ok_or_else`
  when building the error allocates.
- **`collect()` into a `Result` stops at the first error.** `flatten()` and
  `filter_map(Result::ok)` drop errors without a word.
- **`main` returning `Err(e)` prints `Error: {e:?}`**: `Debug`, not
  `Display`, then exits with code 1. anyhow's `Debug` is the readable report.
- **`Box<dyn Error>` isn't `Send`.** Spell it `Box<dyn Error + Send + Sync>`.
- **A guarded `unwrap` is a promise the compiler can't check.** `"".chars().all(..)`
  is true, and all digits can still overflow. Let `parse` be the check.
- **`unwrap`/`expect` are fine** in tests, prototypes, on literals, and on
  invariants where failure means a bug. Prefer `expect("why it can't fail")`.

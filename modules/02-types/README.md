# 02 · Modeling with types

Rust's type system is a design tool, not only a safety net. Put the rules
of your domain into types and the compiler enforces them everywhere, for
free, at every refactoring. This module drills the patterns that make that
work.

## The model in five lines

1. An **enum** is a closed set of shapes, each carrying its own data. A
   `match` must cover all of them, so adding a variant is a checklist.
2. **Patterns** take values apart: nested, sliced, ranged, bound with `@`,
   combined with `|`. They work in `match`, `if let`, `let ... else`,
   `matches!`, function parameters and `for` loops.
3. Matching through a reference **borrows**: `&Option<String>` gives you
   `&String`.
4. A **newtype** (`struct Meters(f64)`) costs nothing at runtime and stops
   values from mixing. With a private field and a parsing constructor it
   also guarantees validity.
5. **Make illegal states unrepresentable**: if a combination of fields is
   nonsense, change the types so it can't be written.

## Smell → tool

| Smell | Reach for |
|---|---|
| A `_` arm on your own enum | Name the variants, so new ones become E0004 |
| Index arithmetic on a slice (`v[0]`, `v[v.len() - 1]`) | Slice patterns: `[first, .., last]`, `[head, rest @ ..]` |
| `x if x >= 1 && x <= 9` | Range pattern with a binding: `x @ 1..=9` |
| `match` that returns `true`/`false` | `matches!(value, Pattern \| Other if guard)` |
| `match` whose other arm only bails out | `let Pattern = expr else { return ... };` |
| `if let` inside `if let` inside `if` | Let chains: `if let A = a && let B = b && cond` (1.88, edition 2024) |
| `opt.is_some()` then `opt.unwrap()` | `if let`, `?`, `map`, `and_then`, `zip`, `ok_or` |
| Bare `f64`/`String` with an implied unit or rule | Newtype; validate once in `parse`, keep the field private |
| `bool` flags plus `Option` fields that go together | One enum whose variants carry their own data |
| A runtime check that methods are called in order | Typestate: one type per state, transitions take `self` |

## Patterns cheat sheet

```rust
match input {
    Input::Key { key: Key::Char('s'), mods: CTRL } => save(),   // nested, const
    Input::Key { key: Key::Esc | Key::F(10), .. } => quit(),     // | inside a field
    Input::Key { key: Key::Char(d @ '1'..='9'), mods: ALT } => tab(d),
    Input::Click { x, y: 0 } => menu(x),                         // literal field
    Input::Resize { width: w @ 20.., height: h @ 5.. } => redraw(w, h),
    _ => {}
}

match segments.as_slice() {
    [] => home(),
    ["users", id] => user(id),
    ["static", rest @ ..] if !rest.is_empty() => asset(rest.join("/")),
    [init @ .., last] => other(init, last),
}

let Some((key, value)) = line.split_once('=') else { continue };
if let Some(raw) = env && let Ok(port) = raw.parse::<u16>() && port >= 1024 { .. }

cert.zip(key).map(|(c, k)| Tls::new(c, k))       // both or nothing
env.and_then(|raw| raw.parse().ok()).filter(|&p| p >= 1024)
nickname.as_deref().unwrap_or(&name)              // &Option<String> → Option<&str>
opt.ok_or(Error::Missing)?                        // Option → Result
```

## Typestate in brief

```rust
struct Release<S> { version: String, state: PhantomData<S> }
impl Release<Tested>   { fn approve(self) -> Release<Approved> { .. } }
impl Release<Approved> { fn publish(self) { .. } }
impl<S> Release<S>     { fn version(&self) -> &str { .. } }  // every state
```

## Gotchas this module drills

- **A lowercase name in a pattern is a new binding**, not a comparison:
  `match x { limit => .. }` matches everything. Use a `const` (its type
  must derive `PartialEq`) or a guard.
- **`x @ 1 | 2` is `(x @ 1) | 2`** (E0408). Write `x @ (1 | 2)`.
- **Guards don't count for exhaustiveness.** `x if x < 0` plus
  `x if x >= 0` still needs a `_`.
- **`[first, .., last]` needs two elements.** A one-element slice falls
  through to the next arm.
- **Matching a field behind `&self` by value tries to move it** (E0507):
  write `match &self.name`, not `match self.name`. And in edition 2024,
  `ref`, `mut` and `&` are errors inside a pattern that is already
  borrowing implicitly.
- **`let ... else` must diverge.** The `else` block has to `return`,
  `break`, `continue` or panic.
- **`#[non_exhaustive]` only affects other crates.** Inside the defining
  crate you can still match exhaustively. Downstream crates must add a `_`
  arm, so you can add variants (or fields) without a breaking change. A
  one-file exercise can't show that, so there's no exercise for it.
- **`Option<NonZeroU32>` is 4 bytes**, the same as `u32`: the value that
  can't occur, zero, represents `None`. `Option<&T>`, `Option<Box<T>>` and
  enums with unused bit patterns get the same "niche" optimization.

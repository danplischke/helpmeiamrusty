# 00 · Warm-up & syntax speedrun

A quick pass over the syntax and tooling you'll use in every module. If the
quiz goes well, the exercises should take a few minutes each.

## Tooling refresh

| Command | What it's for |
|---|---|
| `cargo check` | Type-check without building. The fastest feedback loop. |
| `cargo clippy` | Lints for idioms and common mistakes. This course runs it on every exercise. |
| `cargo fmt` | rustfmt. Formatting is a solved problem; let it decide. |
| `cargo test name_fragment` | Run only the tests whose names contain `name_fragment`. |
| `cargo doc --open` | Docs for your code *and every dependency*, offline. |
| `rustup doc --std` | The standard library docs, offline. |
| `rustc --explain E0502` | The long explanation of any error code. |

rust-analyzer gives you inline types, "fill match arms", "extract function"
and quick fixes for many compiler errors. In VS Code, it reads `rusty/` and
`work/` as two projects; run "rust-analyzer: Reload Workspace" once `work/`
exists.

**Edition 2024** (Rust 1.85+) is the default for new projects. You'll notice:
let chains (`if let Some(x) = a && x > 0`, Rust 1.88), `impl Trait` return
types capturing all lifetimes in scope by default, `unsafe extern` blocks, and
`gen` reserved as a keyword.

## Syntax you'll want back in your fingers

```rust
let x = 5;                  // immutable by default
let mut y = 5;              // opt in to mutation
let x = x.to_string();      // shadowing: a new binding, may change type
const MAX: u32 = 100;       // compile-time constant, type required

let label = if n > 0 { "pos" } else { "non-pos" };   // if is an expression
let v = loop { break 42; };                           // so is loop
'outer: for row in grid { for c in row { if c == 0 { break 'outer; } } }

match n {
    0 => "zero",
    1..=9 => "small",       // inclusive range pattern
    x if x < 0 => "neg",    // guard
    _ => "big",
};

let Some(v) = opt else { return; };          // let-else
if let Some(v) = opt && v > 3 { /* ... */ }  // let chains (2024)
```

## Gotchas this module drills

- **A trailing semicolon turns an expression into a statement.** The block
  then evaluates to `()`.
- **`len()` counts bytes.** Use `chars().count()` for characters, and slice
  only at `char_indices()` offsets.
- **Integer overflow** is a compile error when the compiler can see it, a
  panic in debug builds and wraps silently in release builds. Use
  `checked_*`/`wrapping_*`/`saturating_*` when overflow is possible.
- **Derived `Ord` compares fields in declaration order.**
- **Privacy is per module.** Children can see their ancestors' private
  items, but nothing else can.

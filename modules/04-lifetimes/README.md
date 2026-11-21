# 04 · Lifetimes in practice

Lifetime annotations don't make anything live longer. They say *where a
returned or stored reference points*, so the compiler can check every caller
against the signature alone, never the body. Most lifetime trouble is a
signature that names the wrong source, or the right one too strictly.

## Elision in three rules

1. Each elided input reference gets its own lifetime.
2. With exactly one input lifetime, every elided output gets it.
3. With `&self` or `&mut self`, every elided output gets `self`'s.

Anything else needs names (E0106). `const` and `static` items are `'static`
already. In a return type, write `Parser<'_>` rather than `Parser`: the
borrow stays visible, and rustc warns about the hidden form
(`mismatched_lifetime_syntaxes`).

## Where does the result point?

| Signature | The result borrows from |
|---|---|
| `fn f(s: &str) -> &str` | `s` (rule 2) |
| `fn f(&self, key: &str) -> &str` | `self` only (rule 3); `key` may die first |
| `fn f<'p>(&self, path: &'p str) -> &'p str` | `path` only; `self` may die first |
| `fn f(&self) -> &'static str` | nothing: literals, statics |
| `impl<'a> Lexer<'a> { fn next(&mut self) -> &'a str }` | the input the lexer borrows, not the lexer |
| `fn f<'a>(x: &'a str, y: &'a str) -> &'a str` | either one, so both must outlive the result |
| `fn f<'r, 'a>(v: &'r [Rec<'a>]) -> &'r Rec<'a>` | the slice, which borrows the text |

## Symptom → fix

| Symptom | Usually means | Fix |
|---|---|---|
| "method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`" | Elision tied the output to `self`; the body returns part of an argument | Name the argument's lifetime in the output |
| An argument you only *read* "does not live long enough" | It shares the result's lifetime | Leave it out of that lifetime |
| E0499 / E0502 on the second `next_token()`, or `impl Iterator` won't compile | Output tied to `&mut self` | Return `&'a T` from the struct's parameter |
| "borrowed as mutable … later used here" after the struct is gone | One lifetime doing two jobs: `&'a mut Vec<&'a T>` | Two parameters: `&'v mut Vec<&'s T>` |
| E0310 / E0309 storing a closure in a `Box<dyn Fn>` | `Box<dyn Fn()>` in a field means `+ 'static` | `Box<dyn Fn() + 'a>` and `impl Fn() + 'a` |
| "argument requires that `x` is borrowed for `'a`" calling a closure on a local | `F: Fn(&'a str) -> &'a str` is one caller-chosen `'a` | `F: Fn(&str) -> &str`, i.e. `for<'x> ..` |
| "returning this value requires that `'1` must outlive `'2`" in a `let` closure | A closure in a `let` isn't inferred as higher-ranked | Pass it straight to the function, or use a `fn` |
| E0502 while an `impl Iterator` returned from `&self` is alive | Edition 2024 captures `&self` | `+ use<..>` listing only what it borrows |
| E0521 "borrowed data escapes outside of function" in `thread::spawn` | `spawn` needs `'static` | `thread::scope`, or move owned data in |

## `'static` is about ownership

- `T: 'static` means "holds no borrow shorter than `'static`". `String`,
  `Vec<u8>`, a `move` closure over owned data: all `'static`, all dropped
  normally. It says nothing about how long a *value* lives.
- `&'static T` is different: a reference valid until the program ends
  (literals, statics, `Box::leak`).
- `Box<dyn Trait>` in a field, alias or signature is `Box<dyn Trait + 'static>`;
  `&'a dyn Trait` is `&'a (dyn Trait + 'a)`. Inside a function body, a `let`
  annotation infers the bound instead.
- `thread::spawn` requires a `'static` closure. `thread::scope` joins its
  threads before returning, so they may borrow, and even return borrows.

## Higher-ranked bounds

```rust
fn f<F: Fn(&str) -> &str>(f: F)              // for<'x> Fn(&'x str) -> &'x str
fn f<'a, F: Fn(&'a str) -> &'a str>(f: F)    // one 'a, picked by the caller
Box<dyn Fn(&str) -> &str>                    // higher-ranked as well
v.sort_by_key(|p| p.name.as_str());          // error: K is fixed, can't borrow the element
v.sort_by(|a, b| a.name.cmp(&b.name));       // fine
```

## Variance, the useful part

- `&'a T` is *covariant*: a longer lifetime works where a shorter one is
  expected, which is why `&'static str` goes anywhere `&'a str` does.
- `&'a mut T` is *invariant* in `T`: a `&mut Vec<&'static str>` can't take a
  `&'short str`, and `&'a mut Vec<&'a str>` welds both lifetimes together.
  `Cell<T>` and `RefCell<T>` are invariant too.
- `Box<dyn Trait + 'a>` is covariant in `'a`: a `'static` one can shrink.

## Edition 2024

```rust
fn ids(&self) -> impl Iterator<Item = u32>               // captures &self
fn ids(&self) -> impl Iterator<Item = u32> + use<>       // captures nothing (1.82)
fn fields<'t>(&self, s: &'t str) -> impl Iterator<Item = &'t str> + use<'t>
```

- `use<..>` is a promise the body must keep: a closure reading `self.sep`
  captures `&self`. Copy the field out first, or pass it by value.
- Temporaries in a block's tail expression now drop *before* the block's
  locals, so `cell.borrow().len()` at the end of a function compiles.
- Temporaries in an `if let` scrutinee now drop before the `else` branch, so
  a `RefCell` borrow or a lock taken there is released in `else`.

## Self-referential structs

A struct can't hold a `String` and a `&str` into that same `String`: there's
no lifetime that means "my own field", and moving the struct could leave the
reference dangling. Instead:

- keep the owner outside: `let text = load(); let doc = Doc::parse(&text);`
- store offsets into owned data, `(start, end)` or `Range<usize>`, and slice
  on demand: `&self.text[start..end]`;
- store indices into a `Vec` instead of references to its elements.

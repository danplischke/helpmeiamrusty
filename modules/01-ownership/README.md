# 01 · Ownership & borrowing

The heart of Rust. Most "fighting the borrow checker" comes down to a handful
of patterns. This module drills each one until the fix is a reflex.

## The model in five lines

1. Every value has exactly one **owner**. When the owner goes out of scope,
   the value is dropped.
2. Assignment and passing by value **move** ownership, unless the type is
   `Copy` (integers, floats, `bool`, `char`, shared references, and tuples or
   arrays of those).
3. You can **borrow**: many `&T` at once, *or* exactly one `&mut T`.
4. A borrow lasts until its **last use** (non-lexical lifetimes), not until
   the end of the block.
5. A reference can never outlive the value it points to.

## Patterns and their fixes

| Symptom | Usually means | Reach for |
|---|---|---|
| E0382 use of moved value | A function took ownership it didn't need | Change the parameter to `&T` / `&[T]` / `&str` |
| E0507 cannot move out of borrowed content | Moving a field or element out of a `&` or `&mut` | Borrow it (`&x.field`), or `Option::take`, `mem::take`, `mem::replace` |
| E0502 / E0499 conflicting borrows | A `&` is still alive when you need `&mut` | Reorder, copy `Copy` values out, finish using the borrow first |
| Mutating a Vec while iterating it | The iterator holds a borrow | `retain`, `iter_mut`, `extract_if`, collect-then-apply |
| `self.method()` conflicts with a field borrow | Methods borrow all of `self` | Use the field directly, or destructure `self` |
| Two `&mut` elements of one slice | The checker can't see `i != j` | `get_disjoint_mut` (1.86), `split_at_mut`, `swap` |
| E0106 missing lifetime specifier | Several reference inputs, so elision can't pick one | Name it: `fn f<'a>(x: &'a str, y: &str) -> &'a str` |
| E0515 returns a reference to a local | The value is created inside the function | Return it owned (`String`, `Vec<T>`) |
| Conditional `return` of a borrow, then a mutation | Borrow checker blind spot (NLL problem case #3) | Entry API, or restructure the lookup |

## Signatures: borrow the most general form

```rust
fn f(s: &str)           // not &String: accepts literals and substrings
fn f(v: &[T])           // not &Vec<T>: accepts arrays and sub-slices
fn f(v: &mut [T])       // when you only modify elements in place
fn f(v: &mut Vec<T>)    // only when you push, remove or resize
fn f(v: Vec<T>)         // only when you really keep it (store it, return it)
```

## Handy tools

- `Option::take()` and `Option::replace(v)`
- `std::mem::{take, replace, swap}`
- `Vec::{retain, retain_mut, drain, extract_if, split_off, swap_remove}`
- `slice::{split_at_mut, get_disjoint_mut, swap, chunks_mut}`
- `HashMap::entry(k).or_insert(..) / or_insert_with(..) / or_default() / and_modify(..)`

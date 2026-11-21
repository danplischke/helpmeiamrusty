# 06 · Iterators & collections

Fluent Rust walks data with iterators: no index arithmetic, no bounds
checks, and code that says *what* it computes rather than *how*. This module
drills the adaptors you reach for daily, the laziness that bites, writing
your own iterators, and the collections beyond `Vec` and `HashMap`.

## The model in five lines

1. `Iterator` has one required method, `next(&mut self) -> Option<Item>`.
   Every adaptor and consumer is built on it.
2. Adaptors are **lazy**: they build a recipe. Nothing runs until a consumer
   (`collect`, `sum`, `count`, `find`, `for`) pulls items, one at a time,
   through the whole chain.
3. Until it's consumed, an iterator keeps borrowing its source, and its
   closures keep whatever they captured.
4. Consumers take the iterator by value: it's single-pass. Iterate the
   collection again instead of trying to reuse the iterator.
5. `for x in c` calls `IntoIterator::into_iter(c)`: `c` yields owned items,
   `&c` yields `&T`, `&mut c` yields `&mut T`.

## Pattern → tool

| You want | Reach for |
|---|---|
| Transform and skip in one step | `filter_map`, with `.ok()` or `?` inside |
| Several outputs per input | `flat_map`; `flatten` for nested iterators or `Option`s |
| A running state | `scan` yields as it goes; `fold` for one final value |
| Stop at the first failure | `take_while`, not `filter`; `skip_while` skips only a leading run |
| Look ahead without consuming | `peekable()`, then `peek` or `next_if` |
| Two sequences side by side | `zip` (stops at the shorter); `chain` to append |
| Count alongside items | `enumerate()`; `(1..).zip(..)` to count from 1 |
| Search from the back | `rfind`, `rposition`, `rev()`, never `filter(..).last()` |
| Neighbours, blocks, runs of a slice | `windows(n)`, `array_windows::<N>()`, `chunks_exact(n)` + `remainder()`, `chunk_by` (1.77) |
| Min or max of floats | `min_by` / `max_by` with `f64::total_cmp` |
| Sort by a borrowed field | `sort_by`, comparing the fields |
| Sort by an expensive key | `sort_by_cached_key` (one key per element) |
| Where `x` goes in a sorted slice | `binary_search` → `Err(i)`, or `partition_point` |
| A sequence from a rule | `successors`, `from_fn`, `repeat_n` (1.82), `once`, `empty` |
| An array built from its indices | `std::array::from_fn` |
| Two collections in one pass | `partition`, `unzip` |
| A queue or sliding window | `VecDeque`: `push_back` + `pop_front` |
| Top k, priority queue | `BinaryHeap` (max-first); wrap in `Reverse` for min-first |
| Keys in a range, in order | `BTreeMap::range(a..b)` |
| Set algebra | `HashSet::{union, intersection, difference}` |

## Your own iterators and collections

```rust
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.0 = self.0.checked_sub(1)?; // nothing left: return None
        Some(self.0 + 1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0 as usize, Some(self.0 as usize)) // exact, so:
    }
}
impl ExactSizeIterator for Countdown {} // .len() for free

// `for x in &shelf` (and the same with `&'a mut` and `IterMut`):
impl<'a> IntoIterator for &'a Shelf {
    type Item = &'a Book;
    type IntoIter = std::slice::Iter<'a, Book>;
    fn into_iter(self) -> Self::IntoIter { self.books.iter() }
}

// `iter.collect::<Shelf>()` and `shelf.extend(iter)`:
impl FromIterator<Book> for Shelf { /* fn from_iter<I: IntoIterator<Item = Book>>(iter: I) -> Self */ }
impl Extend<Book> for Shelf { /* fn extend<I: IntoIterator<Item = Book>>(&mut self, iter: I) */ }

// Borrows `self`, though the signature doesn't say so. Edition 2024
// captures every lifetime in scope; edition 2021 needed `+ '_` here.
fn pages(&self) -> impl Iterator<Item = u32> { self.books.iter().map(|b| b.pages) }
```

## Gotchas this module drills

- **An unconsumed adaptor does nothing.** `v.iter().map(|x| log(x));` never
  calls `log` (rustc warns). Use `for` for side effects; keep `map` pure.
- **Items flow one at a time.** `map` then `filter` runs m1 f1 m2 f2, not
  m1 m2 f1 f2.
- **`take_while` eats the first item that fails.** Through `by_ref()` that
  item is gone; `peekable()` + `next_if` leaves it in place.
- **`zip` stops at the shorter input**, silently.
- **`max_by_key` returns the last of equal maxima**, `min_by_key` the first.
- **`f64` isn't `Ord`**: `max()`, `sort()` and float `*_by_key` don't compile.
- **`sort_by_key`, `sort_by_cached_key` and `dedup_by_key` can't return a
  borrow of the element** ("lifetime may not live long enough"). Compare
  with `sort_by` / `dedup_by` instead.
- **`dedup` only removes consecutive duplicates.** Sort or group first.
- **`windows(0)`, `chunks(0)` and `BTreeMap::range(b..a)` with `b > a` panic.**
- **`impl Iterator` hides the rest**: no `rev()`, `len()` or `clone()`
  unless the signature promises them. Return the concrete type if you can.
- **Collecting pairs into a map keeps the last value** for a repeated key.
- **Never assert on `HashMap`/`HashSet` order.** Collect into a `BTreeMap` or
  `BTreeSet`, or sort.

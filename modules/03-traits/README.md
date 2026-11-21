# 03 · Traits & generics

Traits do most of the work in Rust's type system: they bound generics, they
become trait objects, they define operators and conversions. Most surprises
come from a few rules about which impls may exist and what `dyn` can do.

## The model in five lines

1. A generic function works for every type that meets its **bounds**, and can
   use only what the bounds promise. Each concrete type gets its own compiled
   copy: **static dispatch**.
2. `dyn Trait` is "some type that implements `Trait`", known at runtime: a
   pointer to the data plus a **vtable**. It has no static size, so it lives
   behind `&`, `Box`, `Rc` or `Arc`. Generic parameters are `Sized` unless
   you write `?Sized`.
3. A trait can be used as `dyn Trait` only if every method can be called
   through the vtable: it must be **dyn compatible** (formerly "object safe").
4. `impl Trait for Type` needs the trait or the type to be local to your
   crate (the **orphan rule**), and no two impls may overlap.
5. One impl per type: **associated type**. Many impls per type: **generic
   parameter**.

## Pattern → tool

| You want | Reach for |
|---|---|
| One function for many types, no runtime cost | Bounds: `fn f<T: Display>(x: T)`, or `fn f(x: impl Display)` |
| A collection of different types | `Vec<Box<dyn Trait>>` |
| To return one concrete type without naming it | `-> impl Trait` |
| To return one of several types | `-> Box<dyn Trait>`, or an enum |
| A method that breaks dyn compatibility | `where Self: Sized` on that method |
| A foreign trait for a foreign type | A newtype: `struct Series(Vec<Reading>);` |
| The same behaviour for every `T: Bound` | A blanket impl: `impl<T: Display> Summary for T` |
| Conversions | `From` (you get `Into`), `TryFrom` (you get `TryInto`), `FromStr` (you get `parse`) |
| A parameter that takes `&str` or `String` | `impl AsRef<str>` to read it, `impl Into<String>` to keep it |
| `HashMap<String, V>` lookups by `&str` | `Borrow`, which `String` already implements |
| Ordering by a key, or reversed | Manual `Ord` with `.then(..)`, `PartialOrd` delegating to it |
| Operators on a type that owns heap data | Implement them for references too: `impl Add<&T> for &T` |

## Cheat sheet

```rust
fn largest<T: PartialOrd>(items: &[T]) -> Option<&T>         // bound inline
fn tally<T>(items: &[T]) -> HashMap<T, usize> where T: Eq + Hash + Clone
fn describe<S: Shape + ?Sized>(shape: &S) -> String           // takes &dyn Shape too
fn render(&self) -> String where T: Display                    // bound one method only

trait Channel: fmt::Display {                // supertrait: defaults can print self
    const MAX_CHARS: usize = 160;            // associated const, with a default
    fn deliver(&mut self, line: String);     // required
    fn send(&mut self, text: &str) -> bool { /* uses deliver, {self} */ }
    fn history(&self) -> impl Iterator<Item = &str>;   // impl Trait in traits (1.75)
}
trait Source { type Event; fn next_event(&mut self) -> Option<Self::Event>; }
fn pump<S: Source, H: Handler<S::Event>>(source: &mut S, handler: &mut H)

fn then<F: Filter>(self, next: F) -> Chain<Self, F> where Self: Sized;  // not in the vtable
let named: &dyn Named = boxed_plugin.as_ref();  // upcasting to a supertrait (1.86)

impl PartialOrd for Ticket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
self.priority.cmp(&other.priority).then(other.id.cmp(&self.id))  // second key reversed
```

## Gotchas this module drills

- **`.clone()` on a `&T` without `T: Clone` clones the reference.** The error
  shows up later, as a type with an unexpected `&` in it.
- **`vec![Box::new(a), Box::new(b)]` takes its type from the first element.**
  Annotate `Vec<Box<dyn Trait>>`.
- **A `&Box<dyn Trait>` is not a `&dyn Trait`.** Use `.as_ref()` or `&**b`.
- **What breaks dyn compatibility:** generic methods, `Self` as an argument
  or return type, functions without `self`, associated consts,
  `-> impl Trait`, and a `Sized` supertrait such as `Clone`.
- **A type alias doesn't satisfy the orphan rule; a newtype does.** A local
  type as a trait parameter counts: `impl From<Meters> for f64` is fine.
- **A blanket impl claims every type that matches**, so a specific impl for
  one of them conflicts (E0119). Stable Rust has no specialization.
- **Equal values must hash equally.** `#[derive(Hash)]` next to a
  hand-written `PartialEq` breaks `HashMap` and `HashSet` (clippy:
  `derived_hash_with_manual_eq`).
- **`#[derive]` bounds every type parameter**, even one only used in
  `PhantomData`: `Money<C>` is `Copy` only if `C: Copy`.
- **`f64` is `PartialOrd`, not `Ord`** (NaN): no `.max()` or `.sort()`. Use
  `f64::total_cmp`.
- **Implement `From`, not `Into`.** The blanket impl only goes one way.

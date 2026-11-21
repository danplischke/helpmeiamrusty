//! # Writing your own iterators (reference solution)
//!
//! - `next` is the only required method. A good `size_hint` lets
//!   `collect::<Vec<_>>()` allocate the right size up front, and an exact
//!   one lets you implement `ExactSizeIterator` for `len()`.
//!   `DoubleEndedIterator` unlocks `rev`, `rfind` and `nth_back`.
//! - Counting what's left (`remaining`) instead of tracking two positions
//!   makes "the ends meet" automatic: both ends decrement the same count.
//!   `checked_sub(1)?` is the "anything left?" test and the decrement in
//!   one, as `checked_div(step)` is the zero test and the division.
//! - `successors(first, f)` fits when each item follows from the previous
//!   one, and it ends when `f` returns `None`. `saturating_mul` keeps the
//!   doubling from overflowing (a panic in debug builds).
//! - `from_fn` fits when the state is more than "the previous item". The
//!   closure owns it: here a `Peekable`, whose `next_if` takes the next
//!   item only if it passes the test, leaving it in place otherwise.

use std::iter;

/// Start times, in minutes after midnight, of the `step`-minute slots that
/// fit completely in `start..end`. `Slots::new(540, 600, 15)` yields 540,
/// 555, 570 and 585. A `step` of 0, or an empty range, yields nothing.
#[derive(Debug, Clone)]
struct Slots {
    /// The first slot not yet yielded from the front.
    next: u32,
    step: u32,
    /// How many slots are left to yield, from either end.
    remaining: u32,
}

impl Slots {
    fn new(start: u32, end: u32, step: u32) -> Slots {
        let remaining = end.saturating_sub(start).checked_div(step).unwrap_or(0);
        Slots {
            next: start,
            step,
            remaining,
        }
    }
}

impl Iterator for Slots {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.remaining = self.remaining.checked_sub(1)?;
        let slot = self.next;
        self.next += self.step;
        Some(slot)
    }

    /// Exactly how many slots are left: `(n, Some(n))`.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for Slots {
    /// The last slot not yet yielded. `next` and `next_back` can be mixed:
    /// they meet in the middle and never yield a slot twice.
    fn next_back(&mut self) -> Option<u32> {
        self.remaining = self.remaining.checked_sub(1)?;
        Some(self.next + self.remaining * self.step)
    }
}

/// `len()` comes for free from an exact `size_hint`.
impl ExactSizeIterator for Slots {}

/// Retry delays in milliseconds: `first`, then each one double the one
/// before, but never more than `cap` (not even the first). Infinite:
/// callers use `.take(n)`. Doubling must not overflow, even when `cap` is
/// `u64::MAX`. `backoff(100, 1000)` yields 100, 200, 400, 800, 1000, 1000...
fn backoff(first: u64, cap: u64) -> impl Iterator<Item = u64> {
    iter::successors(Some(first.min(cap)), move |&prev| {
        Some(prev.saturating_mul(2).min(cap))
    })
}

/// Packs job costs, in order, into batches that cost at most `budget`. A
/// batch closes when the next job wouldn't fit. A job that costs more than
/// `budget` on its own gets a batch to itself.
fn batches(costs: Vec<u32>, budget: u32) -> impl Iterator<Item = Vec<u32>> {
    let mut costs = costs.into_iter().peekable();
    iter::from_fn(move || {
        let first = costs.next()?;
        let mut batch = vec![first];
        let mut total = first;
        while let Some(cost) = costs.next_if(|&cost| total.saturating_add(cost) <= budget) {
            total += cost;
            batch.push(cost);
        }
        Some(batch)
    })
}

fn main() {
    let morning = Slots::new(9 * 60, 10 * 60, 20);
    println!(
        "{} slots: {:?}",
        morning.len(),
        morning.clone().collect::<Vec<_>>()
    );
    println!("latest first: {:?}", morning.rev().collect::<Vec<_>>());
    println!(
        "delays: {:?}",
        backoff(250, 4000).take(6).collect::<Vec<_>>()
    );
    println!(
        "batches: {:?}",
        batches(vec![5, 3, 4, 9, 1], 8).collect::<Vec<_>>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yields_slots_that_fit() {
        let all: Vec<u32> = Slots::new(540, 600, 15).collect();
        assert_eq!(all, [540, 555, 570, 585]);
        let all: Vec<u32> = Slots::new(540, 610, 15).collect();
        assert_eq!(all, [540, 555, 570, 585], "a partial slot doesn't count");
    }

    #[test]
    fn yields_nothing_when_nothing_fits() {
        assert_eq!(Slots::new(600, 540, 15).next(), None);
        assert_eq!(Slots::new(540, 550, 15).next(), None);
        assert_eq!(Slots::new(540, 600, 0).next(), None);
        assert_eq!(Slots::new(540, 600, 0).next_back(), None);
    }

    #[test]
    fn knows_its_length() {
        let mut slots = Slots::new(0, 60, 20);
        assert_eq!(slots.len(), 3);
        slots.next();
        assert_eq!(slots.size_hint(), (2, Some(2)));
        slots.next_back();
        assert_eq!(slots.len(), 1);
        assert_eq!(Slots::new(0, 1440, 1).len(), 1440);
    }

    #[test]
    fn runs_backwards() {
        let all: Vec<u32> = Slots::new(540, 600, 15).rev().collect();
        assert_eq!(all, [585, 570, 555, 540]);
    }

    #[test]
    fn ends_meet_in_the_middle() {
        let mut slots = Slots::new(0, 50, 10);
        assert_eq!(slots.next(), Some(0));
        assert_eq!(slots.next_back(), Some(40));
        assert_eq!(slots.next(), Some(10));
        assert_eq!(slots.next_back(), Some(30));
        assert_eq!(slots.next(), Some(20));
        assert_eq!(slots.next_back(), None);
        assert_eq!(slots.next(), None);
    }

    #[test]
    fn backs_off_up_to_the_cap() {
        let delays: Vec<u64> = backoff(100, 1000).take(7).collect();
        assert_eq!(delays, [100, 200, 400, 800, 1000, 1000, 1000]);
        let delays: Vec<u64> = backoff(5000, 1000).take(2).collect();
        assert_eq!(
            delays,
            [1000, 1000],
            "not even the first delay exceeds the cap"
        );
    }

    #[test]
    fn backoff_never_overflows() {
        let delays: Vec<u64> = backoff(1 << 62, u64::MAX).take(4).collect();
        assert_eq!(delays, [1 << 62, 1 << 63, u64::MAX, u64::MAX]);
    }

    #[test]
    fn packs_batches() {
        let packed: Vec<Vec<u32>> = batches(vec![3, 4, 2, 6, 1, 1], 7).collect();
        assert_eq!(packed, [vec![3, 4], vec![2], vec![6, 1], vec![1]]);
    }

    #[test]
    fn oversized_jobs_go_alone() {
        let packed: Vec<Vec<u32>> = batches(vec![10, 2, 3, 12], 5).collect();
        assert_eq!(packed, [vec![10], vec![2, 3], vec![12]]);
        assert_eq!(batches(Vec::new(), 5).next(), None);
    }
}

//! # Everything is an expression (reference solution)
//!
//! - `match` on integers must be exhaustive. `i64::MIN..=-1` and the open
//!   range `10..` cover everything, so no `_` arm is needed.
//! - `loop` is the only loop that can `break` with a value. A
//!   `while n != 1 { .. }` followed by `steps` would be just as good here.
//! - `find_map` + `position` is the iterator version of a labelled break:
//!   `'outer: for .. { for .. { if .. { found = ..; break 'outer; } } }`.
//! - `is_multiple_of` (Rust 1.87) reads better than `n % 2 == 0`, and recent
//!   clippy suggests it.

/// "negative", "zero", "small" (1 through 9) or "large" (10 and up).
fn classify(n: i64) -> &'static str {
    match n {
        i64::MIN..=-1 => "negative",
        0 => "zero",
        1..=9 => "small",
        10.. => "large",
    }
}

/// How many steps the Collatz sequence needs to reach 1: halve even numbers,
/// turn odd ones into `3n + 1`. `collatz_steps(1) == 0`.
fn collatz_steps(mut n: u64) -> u32 {
    let mut steps = 0;
    loop {
        if n == 1 {
            break steps;
        }
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        steps += 1;
    }
}

/// Row and column of the first cell equal to `target`, scanning row by row.
fn find_in_grid(grid: &[Vec<i32>], target: i32) -> Option<(usize, usize)> {
    grid.iter().enumerate().find_map(|(row, cells)| {
        cells
            .iter()
            .position(|&cell| cell == target)
            .map(|col| (row, col))
    })
}

fn main() {
    for n in [-3, 0, 7, 42] {
        println!("{n} is {}", classify(n));
    }
    println!("27 takes {} Collatz steps", collatz_steps(27));
    let grid = vec![vec![1, 2, 3], vec![4, 5, 6]];
    println!("5 is at {:?}", find_in_grid(&grid, 5));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies() {
        assert_eq!(classify(-5), "negative");
        assert_eq!(classify(0), "zero");
        assert_eq!(classify(1), "small");
        assert_eq!(classify(9), "small");
        assert_eq!(classify(10), "large");
        assert_eq!(classify(i64::MIN), "negative");
    }

    #[test]
    fn counts_collatz_steps() {
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(collatz_steps(6), 8);
        assert_eq!(collatz_steps(27), 111);
    }

    #[test]
    fn finds_first_match_row_by_row() {
        let grid = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(find_in_grid(&grid, 5), Some((1, 1)));
        assert_eq!(find_in_grid(&grid, 9), None);
        let twice = vec![vec![0, 7], vec![7, 0]];
        assert_eq!(find_in_grid(&twice, 7), Some((0, 1)));
        assert_eq!(find_in_grid(&[], 1), None);
    }
}

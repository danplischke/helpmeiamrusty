//! # Everything is an expression
//!
//! `if`, `match`, `loop` and plain blocks all produce values. Implement the
//! functions below as expressions where you can; you shouldn't need `return`.

/// "negative", "zero", "small" (1 through 9) or "large" (10 and up).
fn classify(n: i64) -> &'static str {
    todo!("classify {n}: a `match` with range patterns")
}

/// How many steps the Collatz sequence needs to reach 1: halve even numbers,
/// turn odd ones into `3n + 1`. `collatz_steps(1) == 0`.
fn collatz_steps(n: u64) -> u32 {
    todo!("collatz_steps({n}): try `loop` with `break value`")
}

/// Row and column of the first cell equal to `target`, scanning row by row.
fn find_in_grid(grid: &[Vec<i32>], target: i32) -> Option<(usize, usize)> {
    todo!("find {target} in {grid:?}: a labelled `break 'outer`, or an iterator chain")
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

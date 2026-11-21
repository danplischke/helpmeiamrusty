//! # Windows, chunks and runs (reference solution)
//!
//! - `windows(n)` yields every overlapping `&[T]` of length `n`, and
//!   nothing when `n` is longer than the slice. It **panics for `n == 0`**,
//!   hence the guard. `chunks(0)` and `chunks_exact(0)` panic too.
//! - `array_windows::<N>()` yields `&[T; N]`, which you can destructure in
//!   the closure's pattern: `[a, b, c]`. Indexing an array with a constant
//!   in range is fine under `indexing_slicing`; indexing a slice isn't.
//! - `chunks_exact(n)` yields only full blocks and keeps the leftovers in
//!   `.remainder()`, which borrows the original slice, not the iterator.
//!   `chunks(n)` instead hands you a short last chunk.
//! - `chunk_by` splits wherever the closure says two neighbours differ. Its
//!   chunks are never empty, so `first()` is always `Some`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
    Rest,
    Fat,
    Cardio,
    Peak,
}

/// The training zone for a heart rate.
fn zone(bpm: u32) -> Zone {
    match bpm {
        0..=99 => Zone::Rest,
        100..=139 => Zone::Fat,
        140..=169 => Zone::Cardio,
        _ => Zone::Peak,
    }
}

/// The average of every `n` consecutive readings (a moving average), one
/// per position. Empty when `n` is 0 or more than the number of readings.
fn moving_average(bpm: &[u32], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    bpm.windows(n)
        .map(|window| f64::from(window.iter().sum::<u32>()) / n as f64)
        .collect()
}

/// Positions of spikes: readings strictly higher than both neighbours. The
/// first and last readings have only one neighbour, so they never count.
fn spikes(bpm: &[u32]) -> Vec<usize> {
    bpm.array_windows()
        .enumerate()
        .filter(|(_, [before, reading, after])| reading > before && reading > after)
        .map(|(i, _)| i + 1)
        .collect()
}

/// The (rounded down) average of each complete block of `n` readings, plus
/// the leftover readings at the end that don't fill a block. `n` is never 0.
fn block_averages(bpm: &[u32], n: usize) -> (Vec<u32>, &[u32]) {
    let blocks = bpm.chunks_exact(n);
    let rest = blocks.remainder();
    let averages = blocks
        .map(|block| block.iter().sum::<u32>() / n as u32)
        .collect();
    (averages, rest)
}

/// Runs of consecutive readings in the same zone, as (zone, how many
/// readings), in order.
fn zone_spans(bpm: &[u32]) -> Vec<(Zone, usize)> {
    bpm.chunk_by(|&a, &b| zone(a) == zone(b))
        .filter_map(|run| run.first().map(|&first| (zone(first), run.len())))
        .collect()
}

fn main() {
    let ride = [92, 118, 131, 150, 147, 158, 171, 166, 135, 104, 98];
    println!("moving average: {:?}", moving_average(&ride, 3));
    println!("spikes at: {:?}", spikes(&ride));
    println!("blocks of 4: {:?}", block_averages(&ride, 4));
    println!("zones: {:?}", zone_spans(&ride));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_averages() {
        assert_eq!(
            moving_average(&[100, 110, 120, 130], 2),
            [105.0, 115.0, 125.0]
        );
        assert_eq!(moving_average(&[100, 110, 120], 3), [110.0]);
        assert_eq!(moving_average(&[100, 110], 1), [100.0, 110.0]);
    }

    #[test]
    fn moving_average_edge_cases() {
        assert!(moving_average(&[100, 110], 3).is_empty());
        assert!(moving_average(&[100, 110], 0).is_empty());
        assert!(moving_average(&[], 1).is_empty());
    }

    #[test]
    fn finds_spikes() {
        assert_eq!(spikes(&[90, 120, 100, 100, 140, 130]), [1, 4]);
        assert!(
            spikes(&[90, 120, 120, 90]).is_empty(),
            "plateaus aren't spikes"
        );
        assert!(spikes(&[150, 100]).is_empty());
        assert!(spikes(&[]).is_empty());
    }

    #[test]
    fn averages_full_blocks() {
        let (averages, rest) = block_averages(&[100, 102, 110, 111, 120, 90, 95], 2);
        assert_eq!(averages, [101, 110, 105]);
        assert_eq!(rest, [95]);
    }

    #[test]
    fn blocks_without_leftovers() {
        let (averages, rest) = block_averages(&[100, 110, 120], 3);
        assert_eq!(averages, [110]);
        assert!(rest.is_empty());
        let (averages, rest) = block_averages(&[100, 110], 3);
        assert!(averages.is_empty());
        assert_eq!(rest, [100, 110]);
    }

    #[test]
    fn spans_of_zones() {
        assert_eq!(
            zone_spans(&[90, 95, 120, 130, 99, 150, 180, 175]),
            [
                (Zone::Rest, 2),
                (Zone::Fat, 2),
                (Zone::Rest, 1),
                (Zone::Cardio, 1),
                (Zone::Peak, 2),
            ]
        );
        assert_eq!(zone_spans(&[145]), [(Zone::Cardio, 1)]);
        assert!(zone_spans(&[]).is_empty());
    }
}

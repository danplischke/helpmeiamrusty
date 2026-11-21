//! # Windows, chunks and runs
//!
//! A heart-rate monitor takes one reading per second. These functions work,
//! but they juggle indices: every `bpm[i]` is a bounds check, a possible
//! panic and an off-by-one waiting to happen. Rewrite them with the slice
//! iterators that hand you the pieces directly: `windows`, `array_windows`,
//! `chunks_exact`, `chunk_by`.
//!
//! Rule: no indexing or slicing (`bpm[i]`, `&bpm[a..b]`). Clippy's
//! `indexing_slicing` lint enforces it.

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
    let mut averages = Vec::new();
    if n == 0 {
        return averages;
    }
    let mut start = 0;
    while start + n <= bpm.len() {
        let mut sum = 0;
        for i in start..start + n {
            sum += bpm[i];
        }
        averages.push(f64::from(sum) / n as f64);
        start += 1;
    }
    averages
}

/// Positions of spikes: readings strictly higher than both neighbours. The
/// first and last readings have only one neighbour, so they never count.
fn spikes(bpm: &[u32]) -> Vec<usize> {
    let mut found = Vec::new();
    for i in 1..bpm.len().saturating_sub(1) {
        if bpm[i] > bpm[i - 1] && bpm[i] > bpm[i + 1] {
            found.push(i);
        }
    }
    found
}

/// The (rounded down) average of each complete block of `n` readings, plus
/// the leftover readings at the end that don't fill a block. `n` is never 0.
fn block_averages(bpm: &[u32], n: usize) -> (Vec<u32>, &[u32]) {
    let full = bpm.len() / n * n;
    let mut averages = Vec::new();
    let mut start = 0;
    while start < full {
        let block = &bpm[start..start + n];
        averages.push(block.iter().sum::<u32>() / n as u32);
        start += n;
    }
    (averages, &bpm[full..])
}

/// Runs of consecutive readings in the same zone, as (zone, how many
/// readings), in order.
fn zone_spans(bpm: &[u32]) -> Vec<(Zone, usize)> {
    let mut spans = Vec::new();
    let mut start = 0;
    for i in 1..=bpm.len() {
        if i == bpm.len() || zone(bpm[i]) != zone(bpm[start]) {
            spans.push((zone(bpm[start]), i - start));
            start = i;
        }
    }
    spans
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

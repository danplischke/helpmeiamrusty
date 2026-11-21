//! # Operators, by value and by reference
//!
//! `a + b` calls `Add::add(a, b)`, which takes both operands *by value*.
//! For a `Copy` type that costs nothing. `Signal` owns a `Vec`, so each
//! operator moves its operands away, and the functions below clone before
//! every one. Implement the operators for references too (`&a + &b`, the
//! way `String + &str` works for strings), then use them so nothing is
//! cloned.
//!
//! Rule: no `.clone()`, and here `#[derive(Clone)]` counts as a call to
//! it. Once nothing clones a `Signal`, drop the derive. Keep the by-value
//! operators: the tests use them.

use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// Audio samples.
#[derive(Debug, Clone, PartialEq, Default)]
struct Signal {
    samples: Vec<f64>,
}

impl Signal {
    fn new(samples: &[f64]) -> Self {
        Self {
            samples: samples.to_vec(),
        }
    }

    /// `self + gain * other`, sample by sample. The result is as long as
    /// the longer signal; past its end, the shorter one counts as silence.
    fn mixed(&self, other: &Signal, gain: f64) -> Signal {
        let len = self.samples.len().max(other.samples.len());
        let at = |signal: &Signal, i: usize| signal.samples.get(i).copied().unwrap_or(0.0);
        Signal {
            samples: (0..len)
                .map(|i| at(self, i) + gain * at(other, i))
                .collect(),
        }
    }
}

/// Plays both signals together.
impl Add for Signal {
    type Output = Signal;

    fn add(self, rhs: Signal) -> Signal {
        self.mixed(&rhs, 1.0)
    }
}

/// Takes `rhs` out of `self`.
impl Sub for Signal {
    type Output = Signal;

    fn sub(self, rhs: Signal) -> Signal {
        self.mixed(&rhs, -1.0)
    }
}

/// Changes the volume.
impl Mul<f64> for Signal {
    type Output = Signal;

    fn mul(self, gain: f64) -> Signal {
        Signal {
            samples: self.samples.iter().map(|sample| sample * gain).collect(),
        }
    }
}

/// Inverts the phase.
impl Neg for Signal {
    type Output = Signal;

    fn neg(self) -> Signal {
        self * -1.0
    }
}

/// Mixes `rhs` into `self`.
impl AddAssign for Signal {
    fn add_assign(&mut self, rhs: Signal) {
        *self = self.mixed(&rhs, 1.0);
    }
}

/// All the tracks played together.
fn mixdown(tracks: &[Signal]) -> Signal {
    let mut mix = Signal::default();
    for track in tracks {
        mix += track.clone();
    }
    mix
}

/// What's left of `a` once `b` is taken out.
fn difference(a: &Signal, b: &Signal) -> Signal {
    a.clone() - b.clone()
}

/// `a` with its phase inverted.
fn inverted(a: &Signal) -> Signal {
    -a.clone()
}

/// Fades from `a` to `b`: `a * (1 - t) + b * t`.
fn crossfade(a: &Signal, b: &Signal, t: f64) -> Signal {
    a.clone() * (1.0 - t) + b.clone() * t
}

fn main() {
    let drums = Signal::new(&[0.5, 0.0, 0.5, 0.0]);
    let bass = Signal::new(&[0.25, 0.25]);
    println!("{:?}", difference(&drums, &bass));
    println!("{:?}", inverted(&bass));
    println!("{:?}", crossfade(&drums, &bass, 0.5));
    println!("{:?}", mixdown(&[drums, bass]));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(samples: &[f64]) -> Signal {
        Signal::new(samples)
    }

    #[test]
    fn operators_by_value() {
        assert_eq!(s(&[1.0, 2.0]) + s(&[0.5]), s(&[1.5, 2.0]));
        assert_eq!(s(&[1.0]) - s(&[0.25, 1.0]), s(&[0.75, -1.0]));
        assert_eq!(s(&[1.0, -2.0]) * 0.5, s(&[0.5, -1.0]));
        assert_eq!(-s(&[1.0, -2.0]), s(&[-1.0, 2.0]));
        let mut mix = s(&[1.0]);
        mix += s(&[1.0, 1.0]);
        assert_eq!(mix, s(&[2.0, 1.0]));
    }

    #[test]
    fn mixdown_adds_every_track() {
        let tracks = [s(&[0.5, 0.5]), s(&[0.25]), s(&[0.0, 0.0, 1.0])];
        assert_eq!(mixdown(&tracks), s(&[0.75, 0.5, 1.0]));
        assert_eq!(mixdown(&[]), Signal::default());
        assert_eq!(tracks[1], s(&[0.25]), "the tracks are only borrowed");
    }

    #[test]
    fn difference_and_inversion() {
        let a = s(&[1.0, 0.5]);
        let b = s(&[0.5, 0.5, 0.25]);
        assert_eq!(difference(&a, &b), s(&[0.5, 0.0, -0.25]));
        assert_eq!(inverted(&a), s(&[-1.0, -0.5]));
        assert_eq!(a, s(&[1.0, 0.5]));
    }

    #[test]
    fn crossfades() {
        let a = s(&[1.0, 1.0]);
        let b = s(&[0.0, 0.5]);
        assert_eq!(crossfade(&a, &b, 0.0), a);
        assert_eq!(crossfade(&a, &b, 0.5), s(&[0.5, 0.75]));
        assert_eq!(crossfade(&a, &b, 1.0), b);
    }
}

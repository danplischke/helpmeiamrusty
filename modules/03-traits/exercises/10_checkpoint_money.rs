//! # Checkpoint: money
//!
//! An amount of money, stored as a whole number of minor units (cents),
//! with the currency as a type parameter: `Money<Eur> + Money<Jpy>` doesn't
//! compile. Each currency is a marker type implementing `Currency`, whose
//! associated consts say how to print and parse it.
//!
//! Implement the trait impls and functions so the tests pass: generic impls
//! with bounds, associated consts, operators, `Display`, `FromStr`,
//! `Default`, and a generic method.

use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use std::str::FromStr;

/// A currency, known at compile time.
///
/// Why the supertraits: `#[derive(Clone, Copy, ..)]` on `Money<C>` only
/// applies when `C` has those traits too, even though no `C` value is ever
/// stored. Requiring them of every `Currency` means that in generic code,
/// any `Money<C>` with `C: Currency` is `Copy`, comparable and printable.
trait Currency: Copy + Ord + fmt::Debug {
    /// "EUR", "JPY".
    const CODE: &'static str;
    /// Digits after the decimal point: 2 for EUR (cents), 0 for JPY.
    const DECIMALS: u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Eur;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Jpy;

impl Currency for Eur {
    const CODE: &'static str = "EUR";
    const DECIMALS: u32 = 2;
}

impl Currency for Jpy {
    const CODE: &'static str = "JPY";
    const DECIMALS: u32 = 0;
}

/// An amount in minor units: `Money::<Eur>::new(1250)` is 12.50 EUR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Money<C> {
    minor: i64,
    currency: PhantomData<C>,
}

impl<C: Currency> Money<C> {
    fn new(minor: i64) -> Self {
        Self {
            minor,
            currency: PhantomData,
        }
    }

    /// Converts at `rate` units of `D` per unit of `C`, rounded to the
    /// nearest minor unit of `D`: 12.50 EUR at 160 JPY per EUR is 2000 JPY.
    fn convert<D: Currency>(self, rate: f64) -> Money<D> {
        todo!("convert {} {} at {rate}", self.minor, C::CODE)
    }
}

/// Zero.
impl<C: Currency> Default for Money<C> {
    fn default() -> Self {
        todo!("zero {}", C::CODE)
    }
}

impl<C: Currency> Add for Money<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        todo!("{} + {}", self.minor, rhs.minor)
    }
}

impl<C: Currency> Sub for Money<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        todo!("{} - {}", self.minor, rhs.minor)
    }
}

impl<C: Currency> Neg for Money<C> {
    type Output = Self;

    fn neg(self) -> Self {
        todo!("-{}", self.minor)
    }
}

/// A price times a quantity.
impl<C: Currency> Mul<i64> for Money<C> {
    type Output = Self;

    fn mul(self, quantity: i64) -> Self {
        todo!("{} * {quantity}", self.minor)
    }
}

impl<C: Currency> AddAssign for Money<C> {
    fn add_assign(&mut self, rhs: Self) {
        todo!("{} += {}", self.minor, rhs.minor)
    }
}

/// "12.50 EUR", "-0.05 EUR", "1250 JPY": a '-' if negative, the major
/// units, a '.' and exactly `DECIMALS` digits (no '.' if `DECIMALS` is 0),
/// then a space and the code.
impl<C: Currency> fmt::Display for Money<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f; // only here to silence "unused variable"
        todo!("format {} {}", self.minor, C::CODE)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseMoneyError;

/// Parses an amount without its code: an optional '-', one or more digits,
/// then, for currencies with decimals only, optionally a '.' and exactly
/// `DECIMALS` digits. So "12.50", "-0.05" and "7" for EUR, "1250" for JPY.
impl<C: Currency> FromStr for Money<C> {
    type Err = ParseMoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!("parse {s:?} as {}", C::CODE)
    }
}

/// The sum of all the prices; zero if there are none.
fn total<C: Currency>(prices: &[Money<C>]) -> Money<C> {
    todo!("add up {} prices in {}", prices.len(), C::CODE)
}

fn main() {
    let coffee: Money<Eur> = "3.20".parse().unwrap_or_default();
    let cake = Money::<Eur>::new(450);
    let bill = total(&[coffee * 2, cake]);
    println!("{bill}");
    println!("{}", bill.convert::<Jpy>(161.5));
    println!("{}", -cake + coffee);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eur(minor: i64) -> Money<Eur> {
        Money::new(minor)
    }

    fn jpy(minor: i64) -> Money<Jpy> {
        Money::new(minor)
    }

    #[test]
    fn arithmetic() {
        assert_eq!(eur(1250) + eur(275), eur(1525));
        assert_eq!(eur(500) - eur(1250), eur(-750));
        assert_eq!(-eur(5), eur(-5));
        assert_eq!(eur(299) * 3, eur(897));
        let mut wallet = jpy(1000);
        wallet += jpy(250);
        assert_eq!(wallet, jpy(1250));
        assert_eq!(Money::<Eur>::default(), eur(0));
    }

    #[test]
    fn money_is_copy_and_ordered() {
        let price = eur(1250);
        let doubled = price + price;
        assert!(price < doubled);
        let prices = [eur(300), eur(1250), eur(99)];
        assert_eq!(prices.iter().max(), Some(&eur(1250)));
    }

    #[test]
    fn totals() {
        assert_eq!(total(&[eur(1250), eur(275), eur(-25)]), eur(1500));
        assert_eq!(total::<Jpy>(&[]), jpy(0));
    }

    #[test]
    fn displays_with_the_right_decimals() {
        assert_eq!(eur(1250).to_string(), "12.50 EUR");
        assert_eq!(eur(7).to_string(), "0.07 EUR");
        assert_eq!(eur(-5).to_string(), "-0.05 EUR");
        assert_eq!(eur(-1250).to_string(), "-12.50 EUR");
        assert_eq!(eur(0).to_string(), "0.00 EUR");
        assert_eq!(jpy(1250).to_string(), "1250 JPY");
        assert_eq!(jpy(-3).to_string(), "-3 JPY");
    }

    #[test]
    fn parses_amounts() {
        assert_eq!("12.50".parse(), Ok(eur(1250)));
        assert_eq!("-0.05".parse(), Ok(eur(-5)));
        assert_eq!("7".parse(), Ok(eur(700)));
        assert_eq!("1250".parse(), Ok(jpy(1250)));
        let amount: Money<Eur> = "-3.20".parse().unwrap();
        assert_eq!(amount.to_string(), "-3.20 EUR");
    }

    #[test]
    fn rejects_malformed_amounts() {
        for bad in ["", "-", "abc", "12.5", "12.345", "1.2.3", "12.", ".50"] {
            assert_eq!(bad.parse::<Money<Eur>>(), Err(ParseMoneyError), "{bad:?}");
        }
        assert_eq!("12.50".parse::<Money<Jpy>>(), Err(ParseMoneyError));
    }

    #[test]
    fn converts_between_currencies() {
        assert_eq!(eur(1250).convert::<Jpy>(160.0), jpy(2000));
        assert_eq!(jpy(1000).convert::<Eur>(0.0062), eur(620));
        assert_eq!(eur(100).convert::<Jpy>(161.55), jpy(162), "rounded");
    }
}

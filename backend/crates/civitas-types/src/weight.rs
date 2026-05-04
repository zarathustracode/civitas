//! Vote weight — wraps `rust_decimal::Decimal` for precise arithmetic.
//!
//! Floating-point arithmetic is unsuitable for vote weights: 0.1 + 0.2 ≠ 0.3
//! is not a property we want a tally to inherit. `Decimal` gives exact base-10
//! arithmetic with sufficient range for any realistic deployment.

use std::ops::{Add, AddAssign, Sub, SubAssign};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A non-negative vote weight.
///
/// `Default` is `1.0` — the standard one-person-one-vote weight. Larger
/// weights arise from delegation accumulation; in v1 there is no other
/// reason for a non-unit base weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-export", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-export", ts(export, type = "string"))]
#[serde(transparent)]
pub struct Weight(pub Decimal);

impl Weight {
    pub const ZERO: Weight = Weight(Decimal::ZERO);
    pub const ONE: Weight = Weight(Decimal::ONE);

    #[must_use]
    pub const fn new(value: Decimal) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn into_inner(self) -> Decimal {
        self.0
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// True if the weight is strictly positive.
    #[must_use]
    pub fn is_positive(self) -> bool {
        self.0 > Decimal::ZERO
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self::ONE
    }
}

impl Add for Weight {
    type Output = Weight;
    fn add(self, rhs: Weight) -> Weight {
        Weight(self.0 + rhs.0)
    }
}

impl AddAssign for Weight {
    fn add_assign(&mut self, rhs: Weight) {
        self.0 += rhs.0;
    }
}

impl Sub for Weight {
    type Output = Weight;
    fn sub(self, rhs: Weight) -> Weight {
        Weight(self.0 - rhs.0)
    }
}

impl SubAssign for Weight {
    fn sub_assign(&mut self, rhs: Weight) {
        self.0 -= rhs.0;
    }
}

impl From<u32> for Weight {
    fn from(n: u32) -> Self {
        Weight(Decimal::from(n))
    }
}

impl From<u64> for Weight {
    fn from(n: u64) -> Self {
        Weight(Decimal::from(n))
    }
}

impl From<Decimal> for Weight {
    fn from(d: Decimal) -> Self {
        Weight(d)
    }
}

impl From<Weight> for Decimal {
    fn from(w: Weight) -> Self {
        w.0
    }
}

impl std::iter::Sum for Weight {
    fn sum<I: Iterator<Item = Weight>>(iter: I) -> Self {
        iter.fold(Weight::ZERO, |acc, w| acc + w)
    }
}

impl std::fmt::Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn default_is_one() {
        assert_eq!(Weight::default(), Weight::ONE);
    }

    #[test]
    fn addition_is_exact() {
        // The classic floating-point trap that doesn't trap us.
        let a = Weight(dec!(0.1));
        let b = Weight(dec!(0.2));
        assert_eq!(a + b, Weight(dec!(0.3)));
    }

    #[test]
    fn add_assign_works() {
        let mut acc = Weight::ZERO;
        acc += Weight::ONE;
        acc += Weight::ONE;
        assert_eq!(acc, Weight::from(2u32));
    }

    #[test]
    fn sum_of_unit_weights() {
        let total: Weight = (0..5).map(|_| Weight::ONE).sum();
        assert_eq!(total, Weight::from(5u32));
    }

    #[test]
    fn round_trip_json() {
        let w = Weight(dec!(3.14));
        let s = serde_json::to_string(&w).unwrap();
        let back: Weight = serde_json::from_str(&s).unwrap();
        assert_eq!(w, back);
    }
}

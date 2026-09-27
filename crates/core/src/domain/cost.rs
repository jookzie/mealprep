use std::{iter::Sum, ops::Add};

use serde::{Deserialize, Serialize};

/// What a composition costs, derived the way its macros are.
///
/// A product without a price adds nothing, so `amount` is a floor rather than a total
/// whenever `complete` is false. The two travel together so that no view can show the
/// figure without the caveat.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Cost {
    pub amount: f64,
    pub complete: bool,
}

impl Cost {
    /// The cost of nothing, which is not missing anything.
    pub const ZERO: Self = Self {
        amount: 0.0,
        complete: true,
    };

    /// The cost of something without a price.
    pub const UNKNOWN: Self = Self {
        amount: 0.0,
        complete: false,
    };
}

impl Add for Cost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            amount: self.amount + other.amount,
            complete: self.complete && other.complete,
        }
    }
}

impl Default for Cost {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Sum for Cost {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_unknown_price_makes_the_sum_incomplete() {
        let priced = Cost {
            amount: 2.5,
            complete: true,
        };
        let total: Cost = [priced, Cost::UNKNOWN, priced].into_iter().sum();
        assert_eq!(
            total,
            Cost {
                amount: 5.0,
                complete: false
            }
        );
    }

    #[test]
    fn nothing_costs_nothing_and_is_complete() {
        let total: Cost = std::iter::empty().sum();
        assert_eq!(total, Cost::ZERO);
    }
}

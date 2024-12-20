use std::fmt;

use crate::errors::{Result, TaError};
use crate::{lit, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Kaufman's Efficiency Ratio (ER).
///
/// It is calculated by dividing the price change over a period by the absolute sum of the price movements that occurred to achieve that change.
/// The resulting ratio ranges between 0.0 and 1.0 with higher values representing a more efficient or trending market.
///
/// # Parameters
///
/// * _period_ - number of periods (integer greater than 0)
///
#[doc(alias = "ER")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct EfficiencyRatio {
    period: usize,
    index: usize,
    count: usize,
    deque: Box<[rust_decimal::Decimal]>,
}

impl EfficiencyRatio {
    /// # Errors
    ///
    /// Will return `Err` if any of the periods is 0
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                index: 0,
                count: 0,
                deque: vec![lit!(0.0); period].into_boxed_slice(),
            }),
        }
    }
}

impl Period for EfficiencyRatio {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<rust_decimal::Decimal> for EfficiencyRatio {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: rust_decimal::Decimal) -> rust_decimal::Decimal {
        let first = if self.count >= self.period {
            self.deque[self.index]
        } else {
            self.count += 1;
            self.deque[0]
        };
        self.deque[self.index] = input;

        self.index = if self.index + 1 < self.period {
            self.index + 1
        } else {
            0
        };

        let mut volatility = lit!(0.0);
        let mut previous = first;
        for n in &self.deque[self.index..self.count] {
            volatility += (previous - n).abs();
            previous = *n;
        }
        for n in &self.deque[0..self.index] {
            volatility += (previous - n).abs();
            previous = *n;
        }

        if volatility == lit!(0.0) {
            lit!(1.0)
        } else {
            (first - input).abs() / volatility
        }
    }
}

impl<T: Close> Next<&T> for EfficiencyRatio {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: &T) -> rust_decimal::Decimal {
        self.next(input.close())
    }
}

impl Reset for EfficiencyRatio {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        for i in 0..self.period {
            self.deque[i] = lit!(0.0);
        }
    }
}

impl Default for EfficiencyRatio {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for EfficiencyRatio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ER({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(EfficiencyRatio);

    #[test]
    fn test_new() {
        assert!(EfficiencyRatio::new(0).is_err());
        assert!(EfficiencyRatio::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut er = EfficiencyRatio::new(3).unwrap();

        assert_eq!(round(er.next(lit!(3.0))), lit!(1.0));
        assert_eq!(round(er.next(lit!(5.0))), lit!(1.0));
        assert_eq!(round(er.next(lit!(2.0))), lit!(0.2));
        assert_eq!(round(er.next(lit!(3.0))), lit!(0.0));
        assert_eq!(round(er.next(lit!(1.0))), lit!(0.667));
        assert_eq!(round(er.next(lit!(3.0))), lit!(0.2));
        assert_eq!(round(er.next(lit!(4.0))), lit!(0.2));
        assert_eq!(round(er.next(lit!(6.0))), lit!(1.0));
    }

    #[test]
    fn test_reset() {
        let mut er = EfficiencyRatio::new(3).unwrap();

        er.next(lit!(3.0));
        er.next(lit!(5.0));

        er.reset();

        assert_eq!(round(er.next(lit!(3.0))), lit!(1.0));
        assert_eq!(round(er.next(lit!(5.0))), lit!(1.0));
        assert_eq!(round(er.next(lit!(2.0))), lit!(0.2));
        assert_eq!(round(er.next(lit!(3.0))), lit!(0.0));
    }

    #[test]
    fn test_display() {
        let er = EfficiencyRatio::new(17).unwrap();
        assert_eq!(format!("{}", er), "ER(17)");
    }
}

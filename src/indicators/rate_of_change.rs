use std::fmt;

use crate::errors::{Result, TaError};
use crate::{lit, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Rate of Change (ROC)
///
/// # Formula
///
/// ROC = (Price<sub>t</sub> - Price<sub>t-n</sub>) / Price<sub>t-n</sub> * 100
///
/// Where:
///
/// * ROC - current value of Rate of Change indicator
/// * P<sub>t</sub> - price at the moment
/// * P<sub>t-n</sub> - price _n_ periods ago
///
/// # Parameters
///
/// * _period_ - number of periods integer greater than 0
///
/// * [Rate of Change, Wikipedia](https://en.wikipedia.org/wiki/Momentum_(technical_analysis))
///
#[doc(alias = "ROC")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RateOfChange {
    period: usize,
    index: usize,
    count: usize,
    deque: Box<[rust_decimal::Decimal]>,
}

impl RateOfChange {
    /// # Errors
    ///
    /// Will return `Err` if period is 0
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

impl Period for RateOfChange {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<rust_decimal::Decimal> for RateOfChange {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: rust_decimal::Decimal) -> rust_decimal::Decimal {
        let previous = if self.count > self.period {
            self.deque[self.index]
        } else {
            self.count += 1;
            if self.count == 1 {
                input
            } else {
                self.deque[0]
            }
        };
        self.deque[self.index] = input;

        self.index = if self.index + 1 < self.period {
            self.index + 1
        } else {
            0
        };

        (input - previous) / previous * lit!(100.0)
    }
}

impl<T: Close> Next<&T> for RateOfChange {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: &T) -> rust_decimal::Decimal {
        self.next(input.close())
    }
}

impl Default for RateOfChange {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for RateOfChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ROC({})", self.period)
    }
}

impl Reset for RateOfChange {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        for i in 0..self.period {
            self.deque[i] = lit!(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(RateOfChange);

    #[test]
    fn test_new() {
        assert!(RateOfChange::new(0).is_err());
        assert!(RateOfChange::new(1).is_ok());
        assert!(RateOfChange::new(100_000).is_ok());
    }

    #[test]
    fn test_next_f64() {
        let mut roc = RateOfChange::new(3).unwrap();

        assert_eq!(round(roc.next(lit!(10.0))), lit!(0.0));
        assert_eq!(round(roc.next(lit!(10.4))), lit!(4.0));
        assert_eq!(round(roc.next(lit!(10.57))), lit!(5.7));
        assert_eq!(round(roc.next(lit!(10.8))), lit!(8.0));
        assert_eq!(round(roc.next(lit!(10.9))), lit!(4.808));
        assert_eq!(round(roc.next(lit!(10.0))), lit!(-5.393));
    }

    #[test]
    fn test_next_bar() {
        fn bar(close: rust_decimal::Decimal) -> Bar {
            Bar::new().close(close)
        }

        let mut roc = RateOfChange::new(3).unwrap();

        assert_eq!(round(roc.next(&bar(lit!(10.0)))), lit!(0.0));
        assert_eq!(round(roc.next(&bar(lit!(10.4)))), lit!(4.0));
        assert_eq!(round(roc.next(&bar(lit!(10.57)))), lit!(5.7));
    }

    #[test]
    fn test_reset() {
        let mut roc = RateOfChange::new(3).unwrap();

        roc.next(lit!(12.3));
        roc.next(lit!(15.0));

        roc.reset();

        assert_eq!(round(roc.next(lit!(10.0))), lit!(0.0));
        assert_eq!(round(roc.next(lit!(10.4))), lit!(4.0));
        assert_eq!(round(roc.next(lit!(10.57))), lit!(5.7));
    }
}

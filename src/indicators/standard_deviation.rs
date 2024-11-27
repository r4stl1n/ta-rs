use std::fmt;

use crate::errors::{Result, TaError};
use crate::{int, lit, Close, Next, Period, Reset};
use rust_decimal::MathematicalOps;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Standard deviation (SD).
///
/// Returns the standard deviation of the last n values.
///
/// # Formula
///
/// ![SD formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/2845de27edc898d2a2a4320eda5f57e0dac6f650)
///
/// Where:
///
/// * _σ_ - value of standard deviation for N given probes.
/// * _N_ - number of probes in observation.
/// * _x<sub>i</sub>_ - i-th observed value from N elements observation.
///
/// # Parameters
///
/// * _period_ - number of periods (integer greater than 0)
///
#[doc(alias = "SD")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct StandardDeviation {
    period: usize,
    index: usize,
    count: usize,
    m: rust_decimal::Decimal,
    m2: rust_decimal::Decimal,
    deque: Box<[rust_decimal::Decimal]>,
}

impl StandardDeviation {
    /// # Errors
    ///
    /// Will return `Err` if `period` is 0
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                index: 0,
                count: 0,
                m: lit!(0.0),
                m2: lit!(0.0),
                deque: vec![lit!(0.0); period].into_boxed_slice(),
            }),
        }
    }

    pub(super) fn mean(&self) -> rust_decimal::Decimal {
        self.m
    }
}

impl Period for StandardDeviation {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<rust_decimal::Decimal> for StandardDeviation {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        let old_val = self.deque[self.index];
        self.deque[self.index] = input;

        self.index = if self.index + 1 < self.period {
            self.index + 1
        } else {
            0
        };

        if self.count < self.period {
            self.count += 1;
            let delta = input - self.m;
            self.m += delta / int!(self.count);
            let delta2 = input - self.m;
            self.m2 += delta * delta2;
        } else {
            let delta = input - old_val;
            let old_m = self.m;
            self.m += delta / int!(self.period);
            let delta2 = input - self.m + old_val - old_m;
            self.m2 += delta * delta2;
        }
        if self.m2 < lit!(0.0) {
            self.m2 = lit!(0.0);
        }

        (self.m2 / int!(self.count))
            .sqrt()
            .expect("Invalid (probably negative) number sent.")
    }
}

impl<T: Close> Next<&T> for StandardDeviation {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for StandardDeviation {
    fn reset(&mut self) {
        self.index = 0;
        self.count = 0;
        self.m = lit!(0.0);
        self.m2 = lit!(0.0);
        for i in 0..self.period {
            self.deque[i] = lit!(0.0);
        }
    }
}

impl Default for StandardDeviation {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for StandardDeviation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SD({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(StandardDeviation);

    #[test]
    fn test_new() {
        assert!(StandardDeviation::new(0).is_err());
        assert!(StandardDeviation::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut sd = StandardDeviation::new(4).unwrap();
        assert_eq!(sd.next(lit!(10.0)), lit!(0.0));
        assert_eq!(sd.next(lit!(20.0)), lit!(5.0));
        assert_eq!(round(sd.next(lit!(30.0))), lit!(8.165));
        assert_eq!(round(sd.next(lit!(20.0))), lit!(7.071));
        assert_eq!(round(sd.next(lit!(10.0))), lit!(7.071));
        assert_eq!(round(sd.next(lit!(100.0))), lit!(35.355));
    }

    #[test]
    fn test_next_floating_point_error() {
        let mut sd = StandardDeviation::new(6).unwrap();
        assert_eq!(sd.next(lit!(1.872)), lit!(0.0));
        assert_eq!(round(sd.next(lit!(1.0))), lit!(0.436));
        assert_eq!(round(sd.next(lit!(1.0))), lit!(0.411));
        assert_eq!(round(sd.next(lit!(1.0))), lit!(0.378));
        assert_eq!(round(sd.next(lit!(1.0))), lit!(0.349));
        assert_eq!(round(sd.next(lit!(1.0))), lit!(0.325));
        assert_eq!(round(sd.next(lit!(1.0))), lit!(0.0));
    }

    #[test]
    fn test_next_with_bars() {
        fn bar(close: rust_decimal::Decimal) -> Bar {
            Bar::new().close(close)
        }

        let mut sd = StandardDeviation::new(4).unwrap();
        assert_eq!(sd.next(&bar(lit!(10.0))), lit!(0.0));
        assert_eq!(sd.next(&bar(lit!(20.0))), lit!(5.0));
        assert_eq!(round(sd.next(&bar(lit!(30.0)))), lit!(8.165));
        assert_eq!(round(sd.next(&bar(lit!(20.0)))), lit!(7.071));
        assert_eq!(round(sd.next(&bar(lit!(10.0)))), lit!(7.071));
        assert_eq!(round(sd.next(&bar(lit!(100.0)))), lit!(35.355));
    }

    #[test]
    fn test_next_same_values() {
        let mut sd = StandardDeviation::new(3).unwrap();
        assert_eq!(sd.next(lit!(4.2)), lit!(0.0));
        assert_eq!(sd.next(lit!(4.2)), lit!(0.0));
        assert_eq!(sd.next(lit!(4.2)), lit!(0.0));
        assert_eq!(sd.next(lit!(4.2)), lit!(0.0));
    }

    #[test]
    fn test_reset() {
        let mut sd = StandardDeviation::new(4).unwrap();
        assert_eq!(sd.next(lit!(10.0)), lit!(0.0));
        assert_eq!(sd.next(lit!(20.0)), lit!(5.0));
        assert_eq!(round(sd.next(lit!(30.0))), lit!(8.165));

        sd.reset();
        assert_eq!(sd.next(lit!(20.0)), lit!(0.0));
    }

    #[test]
    fn test_default() {
        StandardDeviation::default();
    }

    #[test]
    fn test_display() {
        let sd = StandardDeviation::new(5).unwrap();
        assert_eq!(format!("{}", sd), "SD(5)");
    }
}

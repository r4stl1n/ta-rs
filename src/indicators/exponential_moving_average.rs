use std::fmt;
use crate::errors::{Result, TaError};
use crate::{int, lit, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An exponential moving average (EMA), also known as an exponentially weighted moving average
/// (EWMA).
///
/// It is a type of infinite impulse response filter that applies weighting factors which decrease exponentially.
/// The weighting for each older datum decreases exponentially, never reaching zero.
///
/// # Formula
///
/// ![EMA formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/05d06bdbee2c14031fd91ead6f5f772aec1ec964)
///
/// Where:
///
/// * _EMA<sub>t</sub>_ - is the value of the EMA at any time period _t_.
/// * _EMA<sub>t-1</sub>_ - is the value of the EMA at the previous period _t-1_.
/// * _p<sub>t</sub>_ - is the input value at a time period t.
/// * _α_ - is the coefficient that represents the degree of weighting decrease, a constant smoothing factor between 0 and 1.
///
/// _α_ is calculated with the following formula:
///
/// ![alpha formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/d9f6258e152db0644af548972bd6c50a8becf7ee)
///
/// Where:
///
/// * _period_ - number of periods
///
/// # Parameters
///
/// * _period_ - number of periods (integer greater than 0)
///
/// # Links
///
/// * [Exponential moving average, Wikipedia](https://en.wikipedia.org/wiki/Moving_average#Exponential_moving_average)
///
#[doc(alias = "EMA")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ExponentialMovingAverage {
    period: usize,
    k: rust_decimal::Decimal,
    current: rust_decimal::Decimal,
    is_new: bool,
}

impl ExponentialMovingAverage {
    /// # Errors
    ///
    /// Will return `Err` if `period` is 0
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                k: lit!(2.0) / int!(period + 1),
                current: rust_decimal::Decimal::default(),
                is_new: true,
            }),
        }
    }
}

impl Period for ExponentialMovingAverage {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<rust_decimal::Decimal> for ExponentialMovingAverage {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        if self.is_new {
            self.is_new = false;
            self.current = input;
        } else {
            self.current = self.k * input + (lit!(1.0) - self.k) * self.current;
        }
        self.current
    }
}

impl<T: Close> Next<&T> for ExponentialMovingAverage {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for ExponentialMovingAverage {
    fn reset(&mut self) {
        self.current = rust_decimal::Decimal::default();
        self.is_new = true;
    }
}

impl Default for ExponentialMovingAverage {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for ExponentialMovingAverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(ExponentialMovingAverage);

    #[test]
    fn test_new() {
        assert!(ExponentialMovingAverage::new(0).is_err());
        assert!(ExponentialMovingAverage::new(1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();

        assert_eq!(ema.next(lit!(2.0)), lit!(2.0));
        assert_eq!(ema.next(lit!(5.0)), lit!(3.5));
        assert_eq!(ema.next(lit!(1.0)), lit!(2.25));
        assert_eq!(ema.next(lit!(6.25)), lit!(4.25));

        let mut ema = ExponentialMovingAverage::new(3).unwrap();
        let bar1 = Bar::new().close(2);
        let bar2 = Bar::new().close(5);
        assert_eq!(ema.next(&bar1), lit!(2.0));
        assert_eq!(ema.next(&bar2), lit!(3.5));
    }

    #[test]
    fn test_reset() {
        let mut ema = ExponentialMovingAverage::new(5).unwrap();

        assert_eq!(ema.next(lit!(4.0)), lit!(4.0));
        ema.next(lit!(10.0));
        ema.next(lit!(15.0));
        ema.next(lit!(20.0));
        assert_ne!(ema.next(lit!(4.0)), lit!(4.0));

        ema.reset();
        assert_eq!(ema.next(lit!(4.0)), lit!(4.0));
    }

    #[test]
    fn test_default() {
        ExponentialMovingAverage::default();
    }

    #[test]
    fn test_display() {
        let ema = ExponentialMovingAverage::new(7).unwrap();
        assert_eq!(format!("{}", ema), "EMA(7)");
    }
}

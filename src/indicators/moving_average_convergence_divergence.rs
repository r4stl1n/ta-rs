use std::fmt;

use crate::errors::Result;
use crate::indicators::ExponentialMovingAverage as Ema;
use crate::{Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Moving average converge divergence (MACD).
///
/// The MACD indicator (or "oscillator") is a collection of three time series
/// calculated from historical price data, most often the closing price.
/// These three series are:
///
/// * The MACD series proper
/// * The "signal" or "average" series
/// * The "divergence" series which is the difference between the two
///
/// The MACD series is the difference between a "fast" (short period) exponential
/// moving average (EMA), and a "slow" (longer period) EMA of the price series.
/// The average series is an EMA of the MACD series itself.
///
/// # Formula
///
/// # Parameters
///
/// * _`fast_period`_ - period for the fast EMA. Default is 12.
/// * _`slow_period`_ - period for the slow EMA. Default is 26.
/// * _`signal_period`_ - period for the signal EMA. Default is 9.
///
#[doc(alias = "MACD")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MovingAverageConvergenceDivergence {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
}

impl MovingAverageConvergenceDivergence {
    /// # Errors
    ///
    /// Will return `Err` if any of the periods are 0
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Result<Self> {
        Ok(Self {
            fast_ema: Ema::new(fast_period)?,
            slow_ema: Ema::new(slow_period)?,
            signal_ema: Ema::new(signal_period)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovingAverageConvergenceDivergenceOutput {
    pub macd: rust_decimal::Decimal,
    pub signal: rust_decimal::Decimal,
    pub histogram: rust_decimal::Decimal,
}

impl From<MovingAverageConvergenceDivergenceOutput> for (rust_decimal::Decimal,rust_decimal::Decimal,rust_decimal::Decimal) {
    fn from(mo: MovingAverageConvergenceDivergenceOutput) -> Self {
        (mo.macd, mo.signal, mo.histogram)
    }
}

impl Next<rust_decimal::Decimal> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        let fast_val = self.fast_ema.next(input);
        let slow_val = self.slow_ema.next(input);

        let macd = fast_val - slow_val;
        let signal = self.signal_ema.next(macd);
        let histogram = macd - signal;

        MovingAverageConvergenceDivergenceOutput {
            macd,
            signal,
            histogram,
        }
    }
}

impl<T: Close> Next<&T> for MovingAverageConvergenceDivergence {
    type Output = MovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for MovingAverageConvergenceDivergence {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }
}

impl Default for MovingAverageConvergenceDivergence {
    fn default() -> Self {
        Self::new(12, 26, 9).unwrap()
    }
}

impl fmt::Display for MovingAverageConvergenceDivergence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MACD({}, {}, {})",
            self.fast_ema.period(),
            self.slow_ema.period(),
            self.signal_ema.period()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lit;
    use crate::test_helper::*;
    type Macd = MovingAverageConvergenceDivergence;

    test_indicator!(Macd);

    fn round(nums: (rust_decimal::Decimal,rust_decimal::Decimal,rust_decimal::Decimal)) -> (rust_decimal::Decimal,rust_decimal::Decimal,rust_decimal::Decimal) {
        let n0 = (nums.0 * lit!(100.0)).round() / lit!(100.0);
        let n1 = (nums.1 * lit!(100.0)).round() / lit!(100.0);
        let n2 = (nums.2 * lit!(100.0)).round() / lit!(100.0);
        (n0, n1, n2)
    }

    #[test]
    fn test_new() {
        assert!(Macd::new(0, 1, 1).is_err());
        assert!(Macd::new(1, 0, 1).is_err());
        assert!(Macd::new(1, 1, 0).is_err());
        assert!(Macd::new(1, 1, 1).is_ok());
    }

    #[test]
    fn test_macd() {
        let mut macd = Macd::new(3, 6, 4).unwrap();

        assert_eq!(
            round(macd.next(lit!(2.0)).into()),
            (lit!(0.0), lit!(0.0), lit!(0.0))
        );
        assert_eq!(
            round(macd.next(lit!(3.0)).into()),
            (lit!(0.21), lit!(0.09), lit!(0.13))
        );
        assert_eq!(
            round(macd.next(lit!(4.2)).into()),
            (lit!(0.52), lit!(0.26), lit!(0.26))
        );
        assert_eq!(
            round(macd.next(lit!(7.0)).into()),
            (lit!(1.15), lit!(0.62), lit!(0.54))
        );
        assert_eq!(
            round(macd.next(lit!(6.7)).into()),
            (lit!(1.15), lit!(0.83), lit!(0.32))
        );
        assert_eq!(
            round(macd.next(lit!(6.5)).into()),
            (lit!(0.94), lit!(0.87), lit!(0.07))
        );
    }

    #[test]
    fn test_reset() {
        let mut macd = Macd::new(3, 6, 4).unwrap();

        assert_eq!(
            round(macd.next(lit!(2.0)).into()),
            (lit!(0.0), lit!(0.0), lit!(0.0))
        );
        assert_eq!(
            round(macd.next(lit!(3.0)).into()),
            (lit!(0.21), lit!(0.09), lit!(0.13))
        );

        macd.reset();

        assert_eq!(
            round(macd.next(lit!(2.0)).into()),
            (lit!(0.0), lit!(0.0), lit!(0.0))
        );
        assert_eq!(
            round(macd.next(lit!(3.0)).into()),
            (lit!(0.21), lit!(0.09), lit!(0.13))
        );
    }

    #[test]
    fn test_default() {
        Macd::default();
    }

    #[test]
    fn test_display() {
        let indicator = Macd::new(13, 30, 10).unwrap();
        assert_eq!(format!("{}", indicator), "MACD(13, 30, 10)");
    }
}

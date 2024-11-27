use std::fmt;

use crate::errors::Result;
use crate::indicators::{ExponentialMovingAverage, TrueRange};
use crate::{Close, High, Low, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Average true range (ATR).
///
/// A technical analysis volatility indicator, originally developed by J. Welles Wilder.
/// The average true range is an N-day smoothed moving average of the true range values.
/// This implementation uses exponential moving average.
///
/// # Formula
///
/// ATR(period)<sub>t</sub> = EMA(period) of TR<sub>t</sub>
///
/// Where:
///
/// * _EMA(period)_ - [exponential moving average](struct.ExponentialMovingAverage.html) with smoothing period
/// * _TR<sub>t</sub>_ - [true range](struct.TrueRange.html) for period _t_
///
/// # Parameters
///
/// * _period_ - smoothing period of EMA (integer greater than 0)
///
#[doc(alias = "ATR")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct AverageTrueRange {
    true_range: TrueRange,
    ema: ExponentialMovingAverage,
}

impl AverageTrueRange {
    /// # Errors
    ///
    /// Will return `Err` if period is 0
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            true_range: TrueRange::new(),
            ema: ExponentialMovingAverage::new(period)?,
        })
    }
}

impl Period for AverageTrueRange {
    fn period(&self) -> usize {
        self.ema.period()
    }
}

impl Next<rust_decimal::Decimal> for AverageTrueRange {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        self.ema.next(self.true_range.next(input))
    }
}

impl<T: High + Low + Close> Next<&T> for AverageTrueRange {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: &T) -> Self::Output {
        self.ema.next(self.true_range.next(input))
    }
}

impl Reset for AverageTrueRange {
    fn reset(&mut self) {
        self.true_range.reset();
        self.ema.reset();
    }
}

impl Default for AverageTrueRange {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for AverageTrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ATR({})", self.ema.period())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lit;
    use crate::test_helper::*;

    test_indicator!(AverageTrueRange);

    #[test]
    fn test_new() {
        assert!(AverageTrueRange::new(0).is_err());
        assert!(AverageTrueRange::new(1).is_ok());
    }
    #[test]
    fn test_next() {
        let mut atr = AverageTrueRange::new(3).unwrap();

        let bar1 = Bar::new().high(10).low(lit!(7.5)).close(9);
        let bar2 = Bar::new().high(11).low(9).close(lit!(9.5));
        let bar3 = Bar::new().high(9).low(5).close(8);

        assert_eq!(atr.next(&bar1), lit!(2.5));
        assert_eq!(atr.next(&bar2), lit!(2.25));
        assert_eq!(atr.next(&bar3), lit!(3.375));
    }

    #[test]
    fn test_reset() {
        let mut atr = AverageTrueRange::new(9).unwrap();

        let bar1 = Bar::new().high(10).low(lit!(7.5)).close(9);
        let bar2 = Bar::new().high(11).low(9).close(lit!(9.5));

        atr.next(&bar1);
        atr.next(&bar2);

        atr.reset();
        let bar3 = Bar::new().high(60).low(15).close(51);
        assert_eq!(atr.next(&bar3), lit!(45.0));
    }

    #[test]
    fn test_default() {
        AverageTrueRange::default();
    }

    #[test]
    fn test_display() {
        let indicator = AverageTrueRange::new(8).unwrap();
        assert_eq!(format!("{}", indicator), "ATR(8)");
    }
}

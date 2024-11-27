use std::fmt;

use crate::errors::Result;
use crate::indicators::ExponentialMovingAverage as Ema;
use crate::{lit, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Percentage Price Oscillator (PPO).
///
/// The PPO indicator (or "oscillator") is a collection of three time series
/// calculated from historical price data, most often the closing price.
/// These three series are:
///
/// * The PPO series proper
/// * The "signal" or "average" series
/// * The "divergence" series which is the difference between the two
///
/// The PPO series is the difference between a "fast" (short period) exponential
/// moving average (EMA), and a "slow" (longer period) EMA of the price series.
/// The average series is an EMA of the PPO series itself.
///
/// # Formula
///
/// # Parameters
///
/// * _`fast_period`_ - period for the fast EMA. Default is 12.
/// * _`slow_period`_ - period for the slow EMA. Default is 26.
/// * _`signal_period`_ - period for the signal EMA. Default is 9.
///
#[doc(alias = "PPO")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct PercentagePriceOscillator {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
}

impl PercentagePriceOscillator {
    /// # Errors
    ///
    /// Will return `Err` if any of the periods are 0
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Result<Self> {
        Ok(PercentagePriceOscillator {
            fast_ema: Ema::new(fast_period)?,
            slow_ema: Ema::new(slow_period)?,
            signal_ema: Ema::new(signal_period)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentagePriceOscillatorOutput {
    pub ppo: rust_decimal::Decimal,
    pub signal: rust_decimal::Decimal,
    pub histogram: rust_decimal::Decimal,
}

impl From<PercentagePriceOscillatorOutput> for (rust_decimal::Decimal,rust_decimal::Decimal,rust_decimal::Decimal) {
    fn from(po: PercentagePriceOscillatorOutput) -> Self {
        (po.ppo, po.signal, po.histogram)
    }
}

impl Next<rust_decimal::Decimal> for PercentagePriceOscillator {
    type Output = PercentagePriceOscillatorOutput;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        let fast_val = self.fast_ema.next(input);
        let slow_val = self.slow_ema.next(input);

        let ppo = (fast_val - slow_val) / slow_val * lit!(100.0);
        let signal = self.signal_ema.next(ppo);
        let histogram = ppo - signal;

        PercentagePriceOscillatorOutput {
            ppo,
            signal,
            histogram,
        }
    }
}

impl<T: Close> Next<&T> for PercentagePriceOscillator {
    type Output = PercentagePriceOscillatorOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for PercentagePriceOscillator {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }
}

impl Default for PercentagePriceOscillator {
    fn default() -> Self {
        Self::new(12, 26, 9).unwrap()
    }
}

impl fmt::Display for PercentagePriceOscillator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PPO({}, {}, {})",
            self.fast_ema.period(),
            self.slow_ema.period(),
            self.signal_ema.period()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    type Ppo = PercentagePriceOscillator;

    use rust_decimal::Decimal;

    test_indicator!(Ppo);

    fn round(nums: (Decimal, Decimal, Decimal)) -> (Decimal, Decimal, Decimal) {
        use rust_decimal::prelude::RoundingStrategy::MidpointAwayFromZero;
        (
            nums.0.round_dp_with_strategy(2, MidpointAwayFromZero),
            nums.1.round_dp_with_strategy(2, MidpointAwayFromZero),
            nums.2.round_dp_with_strategy(2, MidpointAwayFromZero),
        )
    }

    #[test]
    fn test_new() {
        assert!(Ppo::new(0, 1, 1).is_err());
        assert!(Ppo::new(1, 0, 1).is_err());
        assert!(Ppo::new(1, 1, 0).is_err());
        assert!(Ppo::new(1, 1, 1).is_ok());
    }

    #[test]
    fn test_next() {
        let mut ppo = Ppo::new(3, 6, 4).unwrap();

        assert_eq!(
            round(ppo.next(lit!(2.0)).into()),
            (lit!(0.0), lit!(0.0), lit!(0.0))
        );
        assert_eq!(
            round(ppo.next(lit!(3.0)).into()),
            (lit!(9.38), lit!(3.75), lit!(5.63))
        );
        assert_eq!(
            round(ppo.next(lit!(4.2)).into()),
            (lit!(18.26), lit!(9.56), lit!(8.71))
        );
        assert_eq!(
            round(ppo.next(lit!(8.0)).into()),
            (lit!(31.70), lit!(18.41), lit!(13.29))
        );
        assert_eq!(
            round(ppo.next(lit!(6.7)).into()),
            (lit!(23.94), lit!(20.63), lit!(3.32))
        );
        assert_eq!(
            round(ppo.next(lit!(6.5)).into()),
            (lit!(16.98), lit!(19.17), lit!(-2.19))
        );
    }

    #[test]
    fn test_reset() {
        let mut ppo = Ppo::new(3, 6, 4).unwrap();

        assert_eq!(
            round(ppo.next(lit!(2.0)).into()),
            (lit!(0.0), lit!(0.0), lit!(0.0))
        );
        assert_eq!(
            round(ppo.next(lit!(3.0)).into()),
            (lit!(9.38), lit!(3.75), lit!(5.63))
        );

        ppo.reset();

        assert_eq!(
            round(ppo.next(lit!(2.0)).into()),
            (lit!(0.0), lit!(0.0), lit!(0.0))
        );
        assert_eq!(
            round(ppo.next(lit!(3.0)).into()),
            (lit!(9.38), lit!(3.75), lit!(5.63))
        );
    }

    #[test]
    fn test_default() {
        Ppo::default();
    }

    #[test]
    fn test_display() {
        let indicator = Ppo::new(13, 30, 10).unwrap();
        assert_eq!(format!("{}", indicator), "PPO(13, 30, 10)");
    }
}

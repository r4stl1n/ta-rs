use std::fmt;

use crate::errors::Result;
use crate::indicators::{AverageTrueRange, ExponentialMovingAverage};
use crate::{int, lit, Close, High, Low, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Keltner Channel (KC).
///
/// A Keltner Channel is an indicator showing the Average True Range (ATR) of a
/// price surrounding a central moving average. The ATR bands are typically
/// shown 'k' times moved away from the moving average.
///
/// # Formula
///
/// See EMA, ATR documentation.
///
/// KC is composed as:
///
///  * _KC<sub>Middle Band</sub>_ - Exponential Moving Average (EMA).
///  * _KC<sub>Upper Band</sub>_ = EMA + ATR of observation * multipler (usually 2.0)
///  * _KC<sub>Lower Band</sub>_ = EMA - ATR of observation * multipler (usually 2.0)
///
/// # Links
///
/// * [Keltner channel, Wikipedia](https://en.wikipedia.org/wiki/Keltner_channel)
///
#[doc(alias = "KC")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct KeltnerChannel {
    period: usize,
    multiplier: rust_decimal::Decimal,
    atr: AverageTrueRange,
    ema: ExponentialMovingAverage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeltnerChannelOutput {
    pub average: rust_decimal::Decimal,
    pub upper: rust_decimal::Decimal,
    pub lower: rust_decimal::Decimal,
}

impl KeltnerChannel {
    /// # Errors
    ///
    /// Will return `Err` if period or multiple is 0
    pub fn new(period: usize, multiplier: rust_decimal::Decimal) -> Result<Self> {
        Ok(Self {
            period,
            multiplier,
            atr: AverageTrueRange::new(period)?,
            ema: ExponentialMovingAverage::new(period)?,
        })
    }

    #[must_use]
    pub fn multiplier(&self) -> rust_decimal::Decimal {
        self.multiplier
    }
}

impl Period for KeltnerChannel {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<rust_decimal::Decimal> for KeltnerChannel {
    type Output = KeltnerChannelOutput;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        let atr = self.atr.next(input);
        let average = self.ema.next(input);

        Self::Output {
            average,
            upper: average + atr * self.multiplier,
            lower: average - atr * self.multiplier,
        }
    }
}

impl<T: Close + High + Low> Next<&T> for KeltnerChannel {
    type Output = KeltnerChannelOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        let typical_price = (input.close() + input.high() + input.low()) / lit!(3.0);

        let average = self.ema.next(typical_price);
        let atr = self.atr.next(input);

        Self::Output {
            average,
            upper: average + atr * self.multiplier,
            lower: average - atr * self.multiplier,
        }
    }
}

impl Reset for KeltnerChannel {
    fn reset(&mut self) {
        self.atr.reset();
        self.ema.reset();
    }
}

impl Default for KeltnerChannel {
    fn default() -> Self {
        Self::new(10, int!(2)).unwrap()
    }
}

impl fmt::Display for KeltnerChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KC({}, {})", self.period, self.multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(KeltnerChannel);

    #[test]
    fn test_new() {
        assert!(KeltnerChannel::new(0, lit!(2.0)).is_err());
        assert!(KeltnerChannel::new(1, lit!(2.0)).is_ok());
        assert!(KeltnerChannel::new(2, lit!(2.0)).is_ok());
    }

    #[test]
    fn test_next() {
        let mut kc = KeltnerChannel::new(3, lit!(2.0)).unwrap();

        let a = kc.next(lit!(2.0));
        let b = kc.next(lit!(5.0));
        let c = kc.next(lit!(1.0));
        let d = kc.next(lit!(6.25));

        assert_eq!(round(a.average), lit!(2.0));
        assert_eq!(round(b.average), lit!(3.5));
        assert_eq!(round(c.average), lit!(2.25));
        assert_eq!(round(d.average), lit!(4.25));

        assert_eq!(round(a.upper), lit!(2.0));
        assert_eq!(round(b.upper), lit!(6.5));
        assert_eq!(round(c.upper), lit!(7.75));
        assert_eq!(round(d.upper), lit!(12.25));

        assert_eq!(round(a.lower), lit!(2.0));
        assert_eq!(round(b.lower), lit!(0.5));
        assert_eq!(round(c.lower), lit!(-3.25));
        assert_eq!(round(d.lower), lit!(-3.75));
    }

    #[test]
    fn test_next_with_data_item() {
        let mut kc = KeltnerChannel::new(3, lit!(2.0)).unwrap();

        let dt1 = Bar::new().low(lit!(1.2)).high(lit!(1.7)).close(lit!(1.3)); // typical_price = 1.4
        let o1 = kc.next(&dt1);
        assert_eq!(round(o1.average), lit!(1.4));
        assert_eq!(round(o1.lower), lit!(0.4));
        assert_eq!(round(o1.upper), lit!(2.4));

        let dt2 = Bar::new().low(lit!(1.3)).high(lit!(1.8)).close(lit!(1.4)); // typical_price = 1.5
        let o2 = kc.next(&dt2);
        assert_eq!(round(o2.average), lit!(1.45));
        assert_eq!(round(o2.lower), lit!(0.45));
        assert_eq!(round(o2.upper), lit!(2.45));

        let dt3 = Bar::new().low(lit!(1.4)).high(lit!(1.9)).close(lit!(1.5)); // typical_price = 1.6
        let o3 = kc.next(&dt3);
        assert_eq!(round(o3.average), lit!(1.525));
        assert_eq!(round(o3.lower), lit!(0.525));
        assert_eq!(round(o3.upper), lit!(2.525));
    }

    #[test]
    fn test_reset() {
        let mut kc = KeltnerChannel::new(5, lit!(2.0)).unwrap();

        let out = kc.next(lit!(3.0));

        assert_eq!(out.average, lit!(3.0));
        assert_eq!(out.upper, lit!(3.0));
        assert_eq!(out.lower, lit!(3.0));

        kc.next(lit!(2.5));
        kc.next(lit!(3.5));
        kc.next(lit!(4.0));

        let out = kc.next(lit!(2.0));

        assert_eq!(round(out.average), lit!(2.914));
        assert_eq!(round(out.upper), lit!(4.864));
        assert_eq!(round(out.lower), lit!(0.963));

        kc.reset();
        let out = kc.next(lit!(3.0));
        assert_eq!(out.average, lit!(3.0));
        assert_eq!(out.lower, lit!(3.0));
        assert_eq!(out.upper, lit!(3.0));
    }

    #[test]
    fn test_default() {
        KeltnerChannel::default();
    }

    #[test]
    fn test_display() {
        let kc = KeltnerChannel::new(10, int!(3)).unwrap();
        assert_eq!(format!("{}", kc), "KC(10, 3)");
    }
}

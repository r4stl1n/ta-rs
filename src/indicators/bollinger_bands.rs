use std::fmt;

use crate::errors::Result;
use crate::indicators::StandardDeviation as Sd;
use crate::{lit, Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Bollinger Bands (BB).
/// (BB).
/// It is a type of infinite impulse response filter that calculates Bollinger Bands using Exponential Moving Average.
/// The Bollinger Bands are represented by Average EMA and standard deviaton that is moved 'k' times away in both directions from calculated average value.
///
/// # Formula
///
/// See SMA, SD documentation.
///
/// BB is composed as:
///
///  * _BB<sub>Middle Band</sub>_ - Simple Moving Average (SMA).
///  * _BB<sub>Upper Band</sub>_ = SMA + SD of observation * multipler (usually 2.0)
///  * _BB<sub>Lower Band</sub>_ = SMA - SD of observation * multipler (usually 2.0)
///
/// # Links
///
/// * [Bollinger Bands, Wikipedia](https://en.wikipedia.org/wiki/Bollinger_Bands)
///
#[doc(alias = "BB")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    multiplier: rust_decimal::Decimal,
    sd: Sd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BollingerBandsOutput {
    pub average: rust_decimal::Decimal,
    pub upper: rust_decimal::Decimal,
    pub lower: rust_decimal::Decimal,
}

impl BollingerBands {
    /// # Errors
    ///
    /// Will return `Err` if period or multiplier is 0
    pub fn new(period: usize, multiplier: rust_decimal::Decimal) -> Result<Self> {
        Ok(Self {
            period,
            multiplier,
            sd: Sd::new(period)?,
        })
    }

    #[must_use]
    pub fn multiplier(&self) -> rust_decimal::Decimal {
        self.multiplier
    }
}

impl Period for BollingerBands {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<rust_decimal::Decimal> for BollingerBands {
    type Output = BollingerBandsOutput;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        let sd = self.sd.next(input);
        let mean = self.sd.mean();

        Self::Output {
            average: mean,
            upper: mean + sd * self.multiplier,
            lower: mean - sd * self.multiplier,
        }
    }
}

impl<T: Close> Next<&T> for BollingerBands {
    type Output = BollingerBandsOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for BollingerBands {
    fn reset(&mut self) {
        self.sd.reset();
    }
}

impl Default for BollingerBands {
    fn default() -> Self {
        Self::new(9, lit!(2.0)).unwrap()
    }
}

impl fmt::Display for BollingerBands {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BB({}, {})", self.period, self.multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(BollingerBands);

    #[test]
    fn test_new() {
        assert!(BollingerBands::new(0, lit!(2.0)).is_err());
        assert!(BollingerBands::new(1, lit!(2.0)).is_ok());
        assert!(BollingerBands::new(2, lit!(2.0)).is_ok());
    }

    #[test]
    fn test_next() {
        let mut bb = BollingerBands::new(3, lit!(2.0)).unwrap();

        let a = bb.next(lit!(2.0));
        let b = bb.next(lit!(5.0));
        let c = bb.next(lit!(1.0));
        let d = bb.next(lit!(6.25));

        assert_eq!(round(a.average), lit!(2.0));
        assert_eq!(round(b.average), lit!(3.5));
        assert_eq!(round(c.average), lit!(2.667));
        assert_eq!(round(d.average), lit!(4.083));

        assert_eq!(round(a.upper), lit!(2.0));
        assert_eq!(round(b.upper), lit!(6.5));
        assert_eq!(round(c.upper), lit!(6.066));
        assert_eq!(round(d.upper), lit!(8.562));

        assert_eq!(round(a.lower), lit!(2.0));
        assert_eq!(round(b.lower), lit!(0.5));
        assert_eq!(round(c.lower), lit!(-0.733));
        assert_eq!(round(d.lower), lit!(-0.395));
    }

    #[test]
    fn test_reset() {
        let mut bb = BollingerBands::new(5, lit!(2.0)).unwrap();

        let out = bb.next(lit!(3.0));

        assert_eq!(out.average, lit!(3.0));
        assert_eq!(out.upper, lit!(3.0));
        assert_eq!(out.lower, lit!(3.0));

        bb.next(lit!(2.5));
        bb.next(lit!(3.5));
        bb.next(lit!(4.0));

        let out = bb.next(lit!(2.0));

        assert_eq!(out.average, lit!(3.0));
        assert_eq!(round(out.upper), lit!(4.414));
        assert_eq!(round(out.lower), lit!(1.586));

        bb.reset();
        let out = bb.next(lit!(3.0));
        assert_eq!(out.average, lit!(3.0));
        assert_eq!(out.upper, lit!(3.0));
        assert_eq!(out.lower, lit!(3.0));
    }

    #[test]
    fn test_default() {
        BollingerBands::default();
    }

    #[test]
    fn test_display() {
        let bb = BollingerBands::new(10, crate::int!(3)).unwrap();
        assert_eq!(format!("{}", bb), "BB(10, 3)");
    }
}

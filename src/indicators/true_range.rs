use std::fmt;

use crate::helpers::max3;
use crate::{lit, Close, High, Low, Next, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The range of a day's trading is simply _high_ - _low_.
/// The true range extends it to yesterday's closing price if it was outside of today's range.
///
/// The true range is the largest of one the following:
///
/// * Most recent period's high minus the most recent period's low
/// * Absolute value of the most recent period's high minus the previous close
/// * Absolute value of the most recent period's low minus the previous close
///
/// # Formula
///
/// TR = max[(high - low), abs(high - close<sub>prev</sub>), abs(low - close<sub>prev</sub>)]
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct TrueRange {
    prev_close: Option<rust_decimal::Decimal>,
}

impl TrueRange {
    pub fn new() -> Self {
        Self { prev_close: None }
    }
}

impl Default for TrueRange {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TRUE_RANGE()")
    }
}

impl Next<rust_decimal::Decimal> for TrueRange {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, input: rust_decimal::Decimal) -> Self::Output {
        let distance = match self.prev_close {
            Some(prev) => (input - prev).abs(),
            None => lit!(0.0),
        };
        self.prev_close = Some(input);
        distance
    }
}

impl<T: High + Low + Close> Next<&T> for TrueRange {
    type Output = rust_decimal::Decimal;

    fn next(&mut self, bar: &T) -> Self::Output {
        let max_dist = match self.prev_close {
            Some(prev_close) => {
                let dist1 = bar.high() - bar.low();
                let dist2 = (bar.high() - prev_close).abs();
                let dist3 = (bar.low() - prev_close).abs();
                max3(dist1, dist2, dist3)
            }
            None => bar.high() - bar.low(),
        };
        self.prev_close = Some(bar.close());
        max_dist
    }
}

impl Reset for TrueRange {
    fn reset(&mut self) {
        self.prev_close = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(TrueRange);

    #[test]
    fn test_next_f64() {
        let mut tr = TrueRange::new();
        assert_eq!(round(tr.next(lit!(2.5))), lit!(0.0));
        assert_eq!(round(tr.next(lit!(3.6))), lit!(1.1));
        assert_eq!(round(tr.next(lit!(3.3))), lit!(0.3));
    }

    #[test]
    fn test_next_bar() {
        let mut tr = TrueRange::new();

        let bar1 = Bar::new().high(10).low(lit!(7.5)).close(9);
        let bar2 = Bar::new().high(11).low(9).close(lit!(9.5));
        let bar3 = Bar::new().high(9).low(5).close(8);

        assert_eq!(tr.next(&bar1), lit!(2.5));
        assert_eq!(tr.next(&bar2), lit!(2.0));
        assert_eq!(tr.next(&bar3), lit!(4.5));
    }

    #[test]
    fn test_reset() {
        let mut tr = TrueRange::new();

        let bar1 = Bar::new().high(10).low(lit!(7.5)).close(9);
        let bar2 = Bar::new().high(11).low(9).close(lit!(9.5));

        tr.next(&bar1);
        tr.next(&bar2);

        tr.reset();
        let bar3 = Bar::new().high(60).low(15).close(51);
        assert_eq!(tr.next(&bar3), lit!(45.0));
    }

    #[test]
    fn test_default() {
        TrueRange::default();
    }

    #[test]
    fn test_display() {
        let indicator = TrueRange::new();
        assert_eq!(format!("{}", indicator), "TRUE_RANGE()");
    }
}

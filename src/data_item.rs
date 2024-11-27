use chrono::{DateTime, Utc};
use crate::errors::{Result, TaError};
use crate::{lit, Close, High, Low, Open, Volume};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Data item is used as an input for indicators.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    datetime: DateTime<Utc>,
    open: rust_decimal::Decimal,
    high: rust_decimal::Decimal,
    low: rust_decimal::Decimal,
    close: rust_decimal::Decimal,
    volume: rust_decimal::Decimal,
}

impl Candle {
    #[must_use]
    pub fn builder() -> CandleBuilder {
        CandleBuilder::new()
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
}

impl Open for Candle {
    fn open(&self) -> rust_decimal::Decimal {
        self.open
    }
}

impl High for Candle {
    fn high(&self) -> rust_decimal::Decimal {
        self.high
    }
}

impl Low for Candle {
    fn low(&self) -> rust_decimal::Decimal {
        self.low
    }
}

impl Close for Candle {
    fn close(&self) -> rust_decimal::Decimal {
        self.close
    }
}

impl Volume for Candle {
    fn volume(&self) -> rust_decimal::Decimal {
        self.volume
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CandleBuilder {
    time: Option<DateTime<Utc>>,
    open: Option<rust_decimal::Decimal>,
    high: Option<rust_decimal::Decimal>,
    low: Option<rust_decimal::Decimal>,
    close: Option<rust_decimal::Decimal>,
    volume: Option<rust_decimal::Decimal>,
}

impl CandleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time(mut self, time: DateTime<Utc>) -> Self {
        self.time = Some(time);
        self
    }

    pub fn open(mut self, val: rust_decimal::Decimal) -> Self {
        self.open = Some(val);
        self
    }

    pub fn high(mut self, val: rust_decimal::Decimal) -> Self {
        self.high = Some(val);
        self
    }

    pub fn low(mut self, val: rust_decimal::Decimal) -> Self {
        self.low = Some(val);
        self
    }

    pub fn close(mut self, val: rust_decimal::Decimal) -> Self {
        self.close = Some(val);
        self
    }

    pub fn volume(mut self, val: rust_decimal::Decimal) -> Self {
        self.volume = Some(val);
        self
    }

    pub fn build(self) -> Result<Candle> {
        if let (Some(time), Some(open), Some(high), Some(low), Some(close), Some(volume)) =
            (self.time, self.open, self.high, self.low, self.close, self.volume)
        {
            // validate
            if low <= open
                && low <= close
                && low <= high
                && high >= open
                && high >= close
                && volume >= lit!(0.0)
            {
                let item = Candle {
                    datetime: time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                };
                Ok(item)
            } else {
                Err(TaError::DataItemInvalid)
            }
        } else {
            Err(TaError::DataItemIncomplete)
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;

    #[test]
    fn test_builder() {
        fn assert_valid(
            (open, high, low, close, volume): (
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
            ),
        ) {
            let result = Candle::builder()
                .time(Utc.timestamp_opt(0, 0).single().unwrap_or_else(Utc::now))
                .open(open)
                .high(high)
                .low(low)
                .close(close)
                .volume(volume)
                .build();

            assert!(result.is_ok());
        }

        fn assert_invalid(
            (open, high, low, close, volume): (
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
            ),
        ) {
            let result = Candle::builder()
                .time(Utc.timestamp_opt(0, 0).single().unwrap_or_else(Utc::now))
                .open(open)
                .high(high)
                .low(low)
                .close(close)
                .volume(volume)
                .build();
            assert_eq!(result, Err(TaError::DataItemInvalid));
        }

        let valid_records = vec![
            // open, high, low , close, volume
            (lit!(20.0), lit!(25.0), lit!(15.0), lit!(21.0), lit!(7500.0)),
            (lit!(10.0), lit!(10.0), lit!(10.0), lit!(10.0), lit!(10.0)),
            (lit!(0.0), lit!(0.0), lit!(0.0), lit!(0.0), lit!(0.0)),
        ];
        for record in valid_records {
            assert_valid(record)
        }

        let invalid_records = vec![
            // open, high, low , close, volume
            (lit!(-1.0), lit!(25.0), lit!(15.0), lit!(21.0), lit!(7500.0)),
            (lit!(20.0), lit!(-1.0), lit!(15.0), lit!(21.0), lit!(7500.0)),
            (lit!(20.0), lit!(25.0), lit!(15.0), lit!(-1.0), lit!(7500.0)),
            (lit!(20.0), lit!(25.0), lit!(15.0), lit!(21.0), lit!(-1.0)),
            (lit!(14.9), lit!(25.0), lit!(15.0), lit!(21.0), lit!(7500.0)),
            (lit!(25.1), lit!(25.0), lit!(15.0), lit!(21.0), lit!(7500.0)),
            (lit!(20.0), lit!(25.0), lit!(15.0), lit!(14.9), lit!(7500.0)),
            (lit!(20.0), lit!(25.0), lit!(15.0), lit!(25.1), lit!(7500.0)),
            (lit!(20.0), lit!(15.0), lit!(25.0), lit!(21.0), lit!(7500.0)),
        ];
        for record in invalid_records {
            assert_invalid(record)
        }
    }
}

use super::{lit, Close, High, Low, Open, Volume};

#[derive(Debug, PartialEq)]
pub struct Bar {
    open: rust_decimal::Decimal,
    high: rust_decimal::Decimal,
    low: rust_decimal::Decimal,
    close: rust_decimal::Decimal,
    volume: rust_decimal::Decimal,
}

impl Bar {
    pub fn new() -> Self {
        Self {
            open: lit!(0.0),
            close: lit!(0.0),
            low: lit!(0.0),
            high: lit!(0.0),
            volume: lit!(0.0),
        }
    }

    //pub fn open<T: Into<rust_decimal::Decimal>>(mut self, val :T ) -> Self {
    //    self.open = val.into();
    //    self
    //}

    pub fn high<T: Into<rust_decimal::Decimal>>(mut self, val: T) -> Self {
        self.high = val.into();
        self
    }

    pub fn low<T: Into<rust_decimal::Decimal>>(mut self, val: T) -> Self {
        self.low = val.into();
        self
    }

    pub fn close<T: Into<rust_decimal::Decimal>>(mut self, val: T) -> Self {
        self.close = val.into();
        self
    }

    pub fn volume<T: Into<rust_decimal::Decimal>>(mut self, val: T) -> Self {
        self.volume = val.into();
        self
    }
}

impl Open for Bar {
    fn open(&self) -> rust_decimal::Decimal {
        self.open
    }
}

impl Close for Bar {
    fn close(&self) -> rust_decimal::Decimal {
        self.close
    }
}

impl Low for Bar {
    fn low(&self) -> rust_decimal::Decimal {
        self.low
    }
}

impl High for Bar {
    fn high(&self) -> rust_decimal::Decimal {
        self.high
    }
}

impl Volume for Bar {
    fn volume(&self) -> rust_decimal::Decimal {
        self.volume
    }
}

pub fn round(num: rust_decimal::Decimal) -> rust_decimal::Decimal {
    use rust_decimal::prelude::RoundingStrategy;
    num.round_dp_with_strategy(3, RoundingStrategy::MidpointAwayFromZero)
}

macro_rules! test_indicator {
    ($i:tt) => {
        #[test]
        fn test_indicator() {
            use crate::lit;
            let bar = Bar::new();

            // ensure Default trait is implemented
            let mut indicator = $i::default();

            // ensure Next<rust_decimal::Decimal> is implemented
            let first_output = indicator.next(lit!(12.3));

            // ensure next accepts &DataItem as well
            indicator.next(&bar);

            // ensure Reset is implemented and works correctly
            indicator.reset();
            assert_eq!(indicator.next(lit!(12.3)), first_output);

            // ensure Display is implemented
            let _ = format!("{}", indicator);
        }
    };
}

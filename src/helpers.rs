use rust_decimal::Decimal;
pub const INFINITY: Decimal = Decimal::MAX;
pub const NEG_INFINITY: Decimal = Decimal::MIN;

#[macro_export]
macro_rules! lit {
        ($e:expr) => {
            ::rust_decimal::Decimal::from_str_exact(stringify!($e)).unwrap()
        };
    }

#[macro_export]
macro_rules! int {
        ($e:expr) => {
            ::rust_decimal::Decimal::new($e.try_into().unwrap(), 0)
        };
    }

/// Returns the largest of 3 given numbers.
pub fn max3(a: rust_decimal::Decimal, b: rust_decimal::Decimal, c: rust_decimal::Decimal) -> rust_decimal::Decimal {
    a.max(b).max(c)
}

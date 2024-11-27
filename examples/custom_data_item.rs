use ta::indicators::TrueRange;
use ta::{Close, High, Low, Next};


// You can create your own data items.
// You may want it for different purposes, e.g.:
// - you data source don't have volume or other fields.
// - you want to skip validation to avoid performance penalty.
struct Item {
    high: rust_decimal::Decimal,
    low: rust_decimal::Decimal,
    close: rust_decimal::Decimal,
}

impl Low for Item {
    fn low(&self) -> rust_decimal::Decimal {
        self.low
    }
}

impl High for Item {
    fn high(&self) -> rust_decimal::Decimal {
        self.high
    }
}

impl Close for Item {
    fn close(&self) -> rust_decimal::Decimal {
        self.close
    }
}

fn main() {
    let mut tr = TrueRange::default();
    let mut reader = csv::Reader::from_path("./examples/data/AMZN.csv").unwrap();

    for record in reader.deserialize() {
        let (date, _open, high, low, close, _volume): (String, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal) =
            record.unwrap();
        let item = Item { high, low, close };
        let val = tr.next(&item);
        println!("{date}: {tr} = {val:2.2}");
    }
}

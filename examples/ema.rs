use ta::indicators::ExponentialMovingAverage as Ema;
use ta::Candle;
use ta::Next;

fn main() {
    let mut ema = Ema::new(9).unwrap();
    let mut reader = csv::Reader::from_path("./examples/data/AMZN.csv").unwrap();

    for record in reader.deserialize() {
        let (date, open, high, low, close, volume): (String, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal) =
            record.unwrap();
        let dt = Candle::builder()
            .open(open)
            .high(high)
            .low(low)
            .close(close)
            .volume(volume)
            .build()
            .unwrap();
        let ema_val = ema.next(&dt);
        println!("{}: {} = {:2.2}", date, ema, ema_val);
    }
}

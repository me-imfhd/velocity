use scylla::transport::errors::QueryError;
use rust_decimal_macros::dec;

use super::{ schema::{ Symbol, Ticker }, scylla_tables::ScyllaTicker, ScyllaDb };

impl Ticker {
    pub fn new(symbol: Symbol) -> Ticker {
        Ticker {
            symbol,
            base_volume: dec!(0.0),
            high_price: dec!(0.0),
            last_price: dec!(0.0),
            low_price: dec!(0.0),
            price_change: dec!(0.0),
            price_change_percent: dec!(0.0),
            quote_volume: dec!(0.0),
        }
    }
    fn to_scylla_ticker(&self ) -> ScyllaTicker {
        ScyllaTicker {
            symbol: self.symbol.to_string(),
            base_volume: self.base_volume.to_string(),
            high_price: self.high_price.to_string(),
            last_price: self.last_price.to_string(),
            low_price: self.low_price.to_string(),
            price_change: self.price_change.to_string(),
            price_change_percent: self.price_change_percent.to_string(),
            quote_volume: self.quote_volume.to_string(),
        }
    }
}

impl ScyllaDb {
   
    pub async fn new_ticker(&self, ticker: Ticker) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.ticker_table (
                symbol,
                base_volume,
                quote_volume,
                price_change,
                price_change_percent,
                high_price,
                low_price,
                last_price
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let ticker = ticker.to_scylla_ticker();
        let res = self.session.query(s, ticker).await?;
        Ok(())
    }
}

use std::{ error::Error, str::FromStr };

use scylla::transport::errors::QueryError;
use rust_decimal_macros::dec;
use rust_decimal::Decimal;
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
    fn to_scylla_ticker(&self) -> ScyllaTicker {
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
impl ScyllaTicker {
    fn from_scylla_ticker(&self) -> Ticker {
        Ticker {
            symbol: self.symbol.to_string(),
            base_volume: Decimal::from_str(&self.base_volume).unwrap(),
            high_price: Decimal::from_str(&self.high_price).unwrap(),
            last_price: Decimal::from_str(&self.last_price).unwrap(),
            low_price: Decimal::from_str(&self.low_price).unwrap(),
            price_change: Decimal::from_str(&self.price_change).unwrap(),
            price_change_percent: Decimal::from_str(&self.price_change_percent).unwrap(),
            quote_volume: Decimal::from_str(&self.quote_volume).unwrap(),
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
    pub async fn get_ticker(&self, symbol: Symbol) -> Result<Ticker, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                symbol,
                base_volume,
                quote_volume,
                price_change,
                price_change_percent,
                high_price,
                low_price,
                last_price
            FROM keyspace_1.ticker_table
            WHERE symbol = ? ;
        "#;
        let res = self.session.query(s, (symbol,)).await?;
        let mut tickers = res.rows_typed::<ScyllaTicker>()?;
        let scylla_ticker = tickers
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Ticker does not exist in db".to_string()))?;
        let ticker = scylla_ticker.from_scylla_ticker();
        Ok(ticker)
    }
    pub async fn update_ticker(&self, ticker: &mut Ticker) -> Result<(), Box<dyn Error>> {
        let ticker = ticker.to_scylla_ticker();
        let s =
            r#"
            UPDATE keyspace_1.ticker_table 
            SET
                base_volume = ?,
                quote_volume = ?,
                price_change = ?,
                price_change_percent = ?,
                high_price = ?,
                low_price = ?,
                last_price = ?
            WHERE symbol = ? ;
        "#;
        let res = self.session.query(s, (
            ticker.base_volume,
            ticker.quote_volume,
            ticker.price_change,
            ticker.price_change_percent,
            ticker.high_price,
            ticker.low_price,
            ticker.last_price,
            ticker.symbol,
        )).await?;
        Ok(())
    }
}

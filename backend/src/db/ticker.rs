use scylla::transport::errors::QueryError;

use super::{schema::{ Symbol, TickerSchema }, ScyllaDb};

impl TickerSchema {
    pub fn new(symbol: Symbol) -> TickerSchema {
        TickerSchema {
            symbol,
            base_volume: 0.0,
            high_price: 0.0,
            last_price: 0.0,
            low_price: 0.0,
            price_change: 0.0,
            price_change_percent: 0.0,
            quote_volume: 0.0,
        }
    }
}

impl ScyllaDb{
    pub async fn new_ticker(&self, ticker: TickerSchema) -> Result<(), QueryError> {
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
        let res = self.session.query(s, ticker).await?;
        Ok(())
    }
}
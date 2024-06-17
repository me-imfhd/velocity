use scylla::transport::errors::QueryError;

use super::{schema::{ Exchange, MarketSchema, Symbol }, ScyllaDb};

impl MarketSchema {
    pub fn new(
        symbol: Symbol,
        max_price: f32,
        min_price: f32,
        tick_size: f32,
        max_quantity: f32,
        min_quantity: f32,
        step_size: f32
    ) -> MarketSchema {
        let exchange = Exchange::from_symbol(symbol.clone());
        MarketSchema {
            symbol,
            base: exchange.base.to_string(),
            quote: exchange.quote.to_string(),
            max_price,
            min_price,
            tick_size,
            max_quantity,
            min_quantity,
            step_size,
        }
    }
}

impl ScyllaDb{
    pub async fn new_market(&self, market: MarketSchema) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.market_table (
                symbol,
                base,
                quote,
                max_price,
                min_price,
                tick_size,
                max_quantity,
                min_quantity,
                step_size
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#;
        let res = self.session.query(s, market).await?;
        Ok(())
    }
}
use rust_decimal::Decimal;
use scylla::transport::errors::QueryError;

use super::{ schema::{ Exchange, Market, Symbol }, scylla_tables::ScyllaMarket, ScyllaDb };

impl Market {
    pub fn new(
        symbol: Symbol,
        max_price: Decimal,
        min_price: Decimal,
        tick_size: Decimal,
        max_quantity: Decimal,
        min_quantity: Decimal,
        step_size: Decimal
    ) -> Market {
        let exchange = Exchange::from_symbol(symbol.clone());
        Market {
            symbol,
            base: exchange.base,
            quote: exchange.quote,
            max_price,
            min_price,
            tick_size,
            max_quantity,
            min_quantity,
            step_size,
        }
    }
    fn to_scylla_market(&self) -> ScyllaMarket {
        ScyllaMarket {
            base: self.base.to_string(),
            max_price: self.max_price.to_string(),
            max_quantity: self.max_quantity.to_string(),
            min_price: self.min_price.to_string(),
            min_quantity: self.min_quantity.to_string(),
            quote: self.quote.to_string(),
            step_size: self.step_size.to_string(),
            symbol: self.symbol.to_string(),
            tick_size: self.tick_size.to_string(),
        }
    }
}

impl ScyllaDb {
    pub async fn new_market(&self, market: Market) -> Result<(), QueryError> {
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
        let market = market.to_scylla_market();
        let res = self.session.query(s, market).await?;
        Ok(())
    }
}

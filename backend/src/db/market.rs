use std::{ error::Error, str::FromStr };

use rust_decimal::Decimal;
use scylla::transport::errors::QueryError;

use super::{ schema::{ Asset, Exchange, Market, Symbol }, scylla_tables::ScyllaMarket, ScyllaDb };

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

impl ScyllaMarket {
    fn from_scylla_market(&self) -> Market {
        Market {
            base: Asset::from_str(&self.base).unwrap(),
            max_price: Decimal::from_str(&self.max_price).unwrap(),
            max_quantity: Decimal::from_str(&self.max_quantity).unwrap(),
            min_price: Decimal::from_str(&self.min_price).unwrap(),
            min_quantity: Decimal::from_str(&self.min_quantity).unwrap(),
            quote: Asset::from_str(&self.quote).unwrap(),
            step_size: Decimal::from_str(&self.step_size).unwrap(),
            symbol: self.symbol.to_string(),
            tick_size: Decimal::from_str(&self.tick_size).unwrap(),
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
    pub async fn get_market(&self, symbol: Symbol) -> Result<Market, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                symbol,
                base,
                quote,
                max_price,
                min_price,
                tick_size,
                max_quantity,
                min_quantity,
                step_size
            FROM keyspace_1.market_table
            WHERE symbol = ? ;
        "#;
        let res = self.session.query(s, (symbol,)).await?;
        let mut markets = res.rows_typed::<ScyllaMarket>()?;
        let scylla_market = markets
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Market does not exist in db".to_string()))?;
        let market = scylla_market.from_scylla_market();
        Ok(market)
    }
    pub async fn update_market(&self, market: &mut Market) -> Result<(), Box<dyn Error>> {
        let market = market.to_scylla_market();
        let s =
            r#"
            UPDATE keyspace_1.market_table 
            SET
                max_price = ?,
                min_price = ?,
                tick_size = ?,
                max_quantity = ?,
                min_quantity = ?,
                step_size = ?
            WHERE 
                symbol = ? AND
                base = ? AND
                quote = ? 
            ;
        "#;
        let res = self.session.query(s, (
            market.max_price,
            market.min_price,
            market.tick_size,
            market.max_quantity,
            market.min_quantity,
            market.step_size,
            market.symbol,
            market.base,
            market.quote,
        )).await?;
        Ok(())
    }
}

use std::{ error::Error, sync::atomic::Ordering };

use rust_decimal::prelude::*;
use scylla::transport::errors::QueryError;

use super::{
    get_epoch_ms,
    schema::{ Price, Quantity, Trade },
    scylla_tables::ScyllaTrade,
    ScyllaDb,
};

impl Trade {
    pub fn new(id: i64, is_market_maker: bool, price: Price, quantity: Quantity) -> Trade {
        let timestamp = get_epoch_ms();
        let quote_quantity = price * quantity;
        Trade {
            id,
            quantity: quantity,
            quote_quantity: quote_quantity,
            is_market_maker,
            price: price,
            timestamp: timestamp as i64,
        }
    }
    fn to_scylla_trade(&self) -> ScyllaTrade {
        ScyllaTrade {
            id: self.id,
            is_market_maker: self.is_market_maker,
            price: self.price.to_string(),
            quantity: self.quantity.to_string(),
            quote_quantity: self.quote_quantity.to_string(),
            timestamp: self.timestamp,
        }
    }
}
impl ScyllaTrade {
    fn from_scylla_trade(&self) -> Trade {
        Trade {
            id: self.id,
            is_market_maker: self.is_market_maker,
            price: Decimal::from_str(&self.price).unwrap(),
            quantity: Decimal::from_str(&self.quantity).unwrap(),
            quote_quantity: Decimal::from_str(&self.quote_quantity).unwrap(),
            timestamp: self.timestamp,
        }
    }
}
impl ScyllaDb {
    pub async fn new_trade(&self, trade: Trade) -> Result<(), QueryError> {
        let s =
            r#"
            INSERT INTO keyspace_1.trade_table (
                id,
                quantity,
                quote_quantity,
                is_market_maker,
                price,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?);
        "#;
        let trade = trade.to_scylla_trade();
        let res = self.session.query(s, trade).await?;
        Ok(())
    }
    pub async fn get_trade(&self, trade_id: i64) -> Result<Trade, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                id,
                quantity,
                quote_quantity,
                is_market_maker,
                price,
                timestamp
            FROM keyspace_1.trade_table
            WHERE id = ? ;
        "#;
        let res = self.session.query(s, (trade_id,)).await?;
        let mut trades = res.rows_typed::<ScyllaTrade>()?;
        let scylla_trade = trades
            .next()
            .transpose()?
            .ok_or(QueryError::InvalidMessage("Trade does not exist in db".to_string()))?;
        let trade = scylla_trade.from_scylla_trade();
        Ok(trade)
    }
}

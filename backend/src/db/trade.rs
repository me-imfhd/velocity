use std::sync::atomic::Ordering;

use rust_decimal::prelude::*;
use scylla::transport::errors::QueryError;

use super::{
    get_epoch_ms,
    schema::{ Price, Quantity, Trade },
    scylla_tables::ScyllaTrade,
    ScyllaDb,
    TRADE_ID,
};

impl Trade {
    pub fn new(is_market_maker: bool, price: Price, quantity: Quantity) -> Trade {
        TRADE_ID.fetch_add(1, Ordering::SeqCst);
        let id = TRADE_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        let quote_quantity = price * quantity;
        Trade {
            id: id as i64,
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
}

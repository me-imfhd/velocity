
use std::sync::atomic::Ordering;

use rust_decimal::prelude::*;
use scylla::transport::errors::QueryError;

use super::{from_f32, get_epoch_ms, mul, schema::{Price, Quantity, TradeSchema}, to_f32, ScyllaDb, TRADE_ID};

impl TradeSchema {
    pub fn new(is_market_maker: bool, price: Price, quantity: Quantity) -> TradeSchema {
        TRADE_ID.fetch_add(1, Ordering::SeqCst);
        let id = TRADE_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        let quote_quantity = mul(price, quantity);
        TradeSchema {
            id: id as i64,
            quantity,
            quote_quantity,
            is_market_maker,
            price,
            timestamp: timestamp as i64,
        }
    }
}

impl ScyllaDb{
    pub async fn new_trade(&self, trade: TradeSchema) -> Result<(), QueryError> {
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
        let res = self.session.query(s, trade).await?;
        Ok(())
    }
}
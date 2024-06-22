use std::error::Error;

use rust_decimal::prelude::*;
use scylla::{ frame::value::Counter, transport::errors::QueryError };

use crate::{ get_epoch_ms, Price, Quantity, ScyllaDb, ScyllaTrade, Symbol, Trade };

impl Trade {
    pub fn new(
        id: i64,
        is_market_maker: bool,
        price: Price,
        quantity: Quantity,
        symbol: Symbol
    ) -> Trade {
        let timestamp = get_epoch_ms();
        let quote_quantity = price * quantity;
        Trade {
            id,
            symbol,
            quantity: quantity,
            quote_quantity: quote_quantity,
            is_market_maker,
            price: price,
            timestamp: timestamp as i64,
        }
    }
    pub fn to_scylla_trade(&self) -> ScyllaTrade {
        ScyllaTrade {
            id: self.id,
            symbol: self.symbol.to_string(),
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
            symbol: self.symbol.to_string(),
            is_market_maker: self.is_market_maker,
            price: Decimal::from_str(&self.price).unwrap(),
            quantity: Decimal::from_str(&self.quantity).unwrap(),
            quote_quantity: Decimal::from_str(&self.quote_quantity).unwrap(),
            timestamp: self.timestamp,
        }
    }
}
impl ScyllaDb {
    pub fn trade_entry_statement(&self) -> &str {
        let s =
            r#"
            INSERT INTO keyspace_1.trade_table (
                id,
                symbol,
                quantity,
                quote_quantity,
                is_market_maker,
                price,
                timestamp
            ) VALUES (?, ?, ?, ?, ?, ?, ?);
        "#;
        s
    }
}

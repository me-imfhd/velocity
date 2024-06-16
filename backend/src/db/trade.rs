
use std::sync::atomic::Ordering;

use rust_decimal::prelude::*;

use super::{get_epoch_ms, schema::{Price, Quantity, TradeSchema}, TRADE_ID};

impl TradeSchema {
    pub fn new(is_market_maker: bool, price: Price, quantity: Quantity) -> TradeSchema {
        TRADE_ID.fetch_add(1, Ordering::SeqCst);
        let id = TRADE_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        let quote_quantity =
            Decimal::from_f32_retain(price).unwrap() * Decimal::from_f32_retain(quantity).unwrap();
        let quote_quantity = Decimal::to_f32(&quote_quantity).unwrap();
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
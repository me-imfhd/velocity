use std::{ collections::HashMap, ops::Deref, sync::atomic::Ordering, time::{SystemTime, UNIX_EPOCH} };
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use scylla::{FromUserType, SerializeRow};
use serde::{ Deserialize, Serialize };
use strum_macros::EnumIter;

use super::{enums::{OrderSideEn, OrderStatusEn, OrderTypeEn}, ORDER_ID};

pub type Id = i64;
pub type Symbol = String;
pub type Quantity = f32; // Convert to decimal before performing calculations
pub type Price = f32;
pub type Asset = String;
pub type OrderType = String;
pub type OrderSide = String;
pub type OrderStatus = String;

#[derive(Debug, Deserialize, Serialize, FromUserType, SerializeRow)]
pub struct OrderSchema {
    pub id: Id,
    pub user_id: Id,
    pub symbol: Symbol,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: i64,
}
impl OrderSchema {
    pub fn new(
        user_id: Id,
        initial_quantity: Quantity,
        order_side: OrderSideEn,
        order_type: OrderTypeEn,
        symbol: Symbol
    ) -> OrderSchema {
        ORDER_ID.fetch_add(1, Ordering::SeqCst);
        let id = ORDER_ID.load(Ordering::SeqCst);
        let timestamp = get_epoch_ms();
        OrderSchema {
            id: id as i64,
            user_id,
            filled_quantity: 0.0,
            initial_quantity,
            order_side: order_side.to_string(),
            order_status: OrderStatusEn::Processing.to_string(),
            order_type: order_type.to_string(),
            symbol,
            timestamp: timestamp as i64,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, FromUserType, )]
pub struct UserSchema {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
#[derive(Debug, Serialize, Deserialize, FromUserType, )]
pub struct TradeSchema {
    id: Id,
    quantity: Quantity,
    quote_quantity: Quantity,
    is_market_maker: bool,
    price: Price,
    timestamp: i64,
}
#[derive(Debug, Deserialize, Serialize, FromUserType, )]
pub struct TickerSchema {
    symbol: Symbol,
    base_volume: f32,
    quote_volume: f32,
    price_change: f32,
    price_change_percent: f32,
    high_price: f32,
    low_price: f32,
    last_price: f32,
}

#[derive(Debug, Deserialize, Serialize, FromUserType, )]
pub struct MarketSchema {
    base: Asset,
    quote: Asset,
    symbol: Symbol,
    max_price: Option<f32>,
    min_price: f32,
    tick_size: f32,
    max_quanity: Option<f32>,
    min_quantity: f32,
    step_size: f32,
}
fn get_epoch_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}
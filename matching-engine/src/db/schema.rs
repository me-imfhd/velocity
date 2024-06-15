use std::{ collections::HashMap, ops::Deref };

use redis::{ from_redis_value, ErrorKind, FromRedisValue, RedisError, Value };
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };

use crate::matching_engine::{
    orderbook::{ OrderSide, OrderType, Price },
    Asset,
    Id,
    Quantity,
    Symbol,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderSchema {
    pub id: Id,
    pub user_id: Id,
    pub symbol: Symbol,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: u128,
}
#[derive(Debug, Deserialize, Serialize)]
pub enum OrderStatus {
    Processing,
    Filled,
    PartiallyFilled,
    Failed,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct UserSchema {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TradeSchema {
    id: Id,
    quantity: Quantity,
    quote_quantity: Quantity,
    is_market_maker: bool,
    price: Price,
    timestamp: u128,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TickerSchema {
    symbol: Symbol,
    base_volume: Decimal,
    quote_volume: Decimal,
    price_change: Decimal,
    price_change_percent: Decimal,
    high_price: Decimal,
    low_price: Decimal,
    last_price: Decimal,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketSchema {
    base: Asset,
    quote: Asset,
    symbol: Symbol,
    filters: Filters,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Filters {
    price: PriceFilter,
    quantity: QuantityFilter,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct PriceFilter {
    max_price: Option<Decimal>,
    min_price: Decimal,
    tick_size: Decimal,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct QuantityFilter {
    max_quanity: Option<Decimal>,
    min_quantity: Decimal,
    step_size: Decimal,
}

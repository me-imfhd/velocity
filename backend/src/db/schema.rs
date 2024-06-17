use std::{
    collections::HashMap,
    ops::Deref,
    sync::atomic::Ordering,
    time::{ SystemTime, UNIX_EPOCH },
};
use bigdecimal::{ FromPrimitive, ToPrimitive };
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use scylla::{ FromRow, SerializeRow };
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{
    enums::{ AssetEn, OrderSideEn, OrderStatusEn, OrderTypeEn },
    get_epoch_ms,
    ORDER_ID,
    TRADE_ID,
    USER_ID,
};

pub type Id = i64;
pub type Symbol = String;
pub type Quantity = f32; // Convert to decimal before performing calculations
pub type Price = f32;
pub type Asset = String;
pub type OrderType = String;
pub type OrderSide = String;
pub type OrderStatus = String;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct Exchange {
    pub base: AssetEn,
    pub quote: AssetEn,
    pub symbol: Symbol,
}

impl Exchange {
    pub fn new(base: AssetEn, quote: AssetEn) -> Exchange {
        let base_string = base.to_string();
        let quote_string = quote.to_string();
        let symbol = format!("{}_{}", base_string, quote_string);
        Exchange {
            base,
            quote,
            symbol,
        }
    }
    pub fn from_symbol(symbol: Symbol) -> Exchange {
        let symbols: Vec<&str> = symbol.split("_").collect();
        let base_str = symbols.get(0).unwrap();
        let quote_str = symbols.get(1).unwrap();
        let base = AssetEn::from_str(&base_str).expect("Incorrect symbol");
        let quote = AssetEn::from_str(&quote_str).expect("Incorrect symbol");
        Exchange::new(base, quote)
    }
}

#[derive(Debug, Deserialize, Serialize, SerializeRow)]
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

#[derive(Debug, Clone, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct UserSchema {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
#[derive(Debug, Serialize, Deserialize, SerializeRow)]
pub struct TradeSchema {
    pub id: Id,
    pub quantity: Quantity,
    pub quote_quantity: Quantity,
    pub is_market_maker: bool,
    pub price: Price,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow)]
pub struct TickerSchema {
    pub symbol: Symbol,
    pub base_volume: f32,
    pub quote_volume: f32,
    pub price_change: f32,
    pub price_change_percent: f32,
    pub high_price: f32,
    pub low_price: f32,
    pub last_price: f32,
}

#[derive(Debug, Deserialize, Serialize, SerializeRow)]
pub struct MarketSchema {
    pub symbol: Symbol,
    pub base: Asset,
    pub quote: Asset,
    pub max_price: f32,
    pub min_price: f32,
    pub tick_size: f32,
    pub max_quantity: f32,
    pub min_quantity: f32,
    pub step_size: f32,
}

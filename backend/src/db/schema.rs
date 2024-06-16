use std::{
    collections::HashMap,
    ops::Deref,
    sync::atomic::Ordering,
    time::{ SystemTime, UNIX_EPOCH },
};
use bigdecimal::{ FromPrimitive, ToPrimitive };
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use scylla::SerializeRow ;
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{ enums::{ AssetEn, OrderSideEn, OrderStatusEn, OrderTypeEn }, ORDER_ID, TRADE_ID, USER_ID };

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

#[derive(Debug, Deserialize, Serialize,  SerializeRow)]
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
#[derive(Debug, Deserialize, Serialize, SerializeRow)]
pub struct UserSchema {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
impl UserSchema {
    pub fn new() -> UserSchema{
        USER_ID.fetch_add(1, Ordering::SeqCst);
        let id = USER_ID.load(Ordering::SeqCst);
        let mut balance: HashMap<Asset, Quantity> = HashMap::new();
        let mut locked_balance: HashMap<Asset, Quantity> = HashMap::new();
        for asset in AssetEn::iter() {
            balance.insert(asset.to_string(), 0.0);
        }
        for asset in AssetEn::iter() {
            locked_balance.insert(asset.to_string(), 0.0);
        }
        UserSchema {
            id: id as i64,
            balance,
            locked_balance,
        }
    }
}
#[derive(Debug, Serialize, Deserialize,  SerializeRow)]
pub struct TradeSchema {
    pub id: Id,
    pub quantity: Quantity,
    pub quote_quantity: Quantity,
    pub is_market_maker: bool,
    pub price: Price,
    pub timestamp: i64,
}
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
#[derive(Debug, Deserialize, Serialize,  SerializeRow)]
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
impl TickerSchema {
    pub fn new(symbol: Symbol) -> TickerSchema{
        TickerSchema{
            symbol,
            base_volume: 0.0,
            high_price: 0.0,
            last_price: 0.0,
            low_price: 0.0,
            price_change: 0.0,
            price_change_percent: 0.0,
            quote_volume: 0.0
        }
    }
}

#[derive(Debug, Deserialize, Serialize,  SerializeRow)]
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
impl MarketSchema {
    pub fn new(
        symbol: Symbol,
        max_price: f32,
        min_price: f32,
        tick_size: f32,
        max_quantity: f32,
        min_quantity: f32,
        step_size: f32
    ) -> MarketSchema {
        let exchange = Exchange::from_symbol(symbol.clone());
        MarketSchema {
            symbol,
            base: exchange.base.to_string(),
            quote: exchange.quote.to_string(),
            max_price,
            min_price,
            tick_size,
            max_quantity,
            min_quantity,
            step_size,
        }
    }
}
fn get_epoch_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

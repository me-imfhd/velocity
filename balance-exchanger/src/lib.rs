#![allow(unused)]
use std::{ collections::HashMap, time::{ SystemTime, UNIX_EPOCH } };
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use scylla::{ FromRow, SerializeRow, Session };
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub mod db;
pub mod user;
pub mod order;
pub mod trade;
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize, EnumStringify)]
pub enum Asset {
    USDT,
    BTC,
    SOL,
    ETH,
}
impl Asset {
    fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in Asset::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
pub type Symbol = String;
pub type Id = i64;
pub type Quantity = Decimal;
pub type Price = Decimal;
#[derive(Debug, Serialize, Deserialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: String,
}
impl Exchange {
    pub fn new(base: Asset, quote: Asset) -> Exchange {
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
        let base = Asset::from_str(&base_str).expect("Incorrect symbol");
        let quote = Asset::from_str(&quote_str).expect("Incorrect symbol");
        Exchange::new(base, quote)
    }
}
pub struct ScyllaDb {
    session: Session,
}
#[derive(Debug, Clone, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaUser {
    pub id: i64,
    pub balance: HashMap<String, String>,
    pub locked_balance: HashMap<String, String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct QueueTrade {
    user_id_1: Id,
    user_id_2: Id,
    exchange: Exchange,
    base_quantity: Quantity,
    price: Price,
    is_market_maker: bool,
    order_status_1: OrderStatus,
    order_status_2: OrderStatus,
    order_id_1: Id,
    order_id_2: Id,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    pub id: Id,
    pub user_id: Id,
    pub symbol: Symbol,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: i64,
}
#[derive(Debug, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: String,
    pub price: String,
    pub initial_quantity: String,
    pub filled_quantity: String,
    pub order_type: String,
    pub order_side: String,
    pub order_status: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize, EnumStringify, EnumIter)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Failed,
}
impl OrderStatus {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderStatus::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, EnumStringify, EnumIter)]
pub enum OrderSide {
    Bid,
    Ask,
}
impl OrderSide {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderSide::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify, EnumIter)]
pub enum OrderType {
    Market,
    Limit,
}
impl OrderType {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderType::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: Id,
    pub symbol: Symbol,
    pub quantity: Quantity,
    pub quote_quantity: Quantity,
    pub is_market_maker: bool,
    pub price: Price,
    pub timestamp: i64,
}
#[derive(Debug, Serialize, Deserialize, SerializeRow, FromRow)]
pub struct ScyllaTrade {
    pub id: i64,
    pub symbol: Symbol,
    pub quantity: String,
    pub quote_quantity: String,
    pub is_market_maker: bool,
    pub price: String,
    pub timestamp: i64,
}
pub fn get_epoch_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

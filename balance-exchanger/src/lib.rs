#![allow(unused)]
use std::collections::HashMap;
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use scylla::{FromRow, SerializeRow, Session};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub mod db;
pub mod user;
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
pub type Id = u64;
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
pub struct ScyllaDb{
    session: Session
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
#[derive(Debug, Deserialize)]
pub struct QueueTrade {
    pub user_id_1: u64,
    pub user_id_2: u64,
    pub exchange: Exchange,
    pub base_quantity: Quantity,
    pub price: Price,
    pub is_market_maker: bool,
}
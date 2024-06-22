#![allow(non_camel_case_types)]
use std::{ cell::Cell, str::FromStr, sync::atomic::{ AtomicU64, Ordering } };
use engine::Exchange;
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use strum_macros::{ EnumIter, EnumString };
pub mod orderbook;
pub mod engine;
pub mod error;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Serialize, Deserialize, EnumStringify)]
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Serialize, Deserialize, EnumStringify)]
pub enum RegisteredSymbols {
    SOL_USDT,
    BTC_USDT,
    ETH_USDT
}
use std::{ cell::Cell, sync::atomic::{ AtomicU64, Ordering } };
use rust_decimal::Decimal;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
pub mod orderbook;
pub mod engine;
pub mod error;
pub mod users;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Asset {
    USDT,
    BTC,
    SOL,
    ETH,
}

pub type Id = u64;
pub type Quantity = Decimal;

pub static ORDER_ID: AtomicU64 = AtomicU64::new(1);
pub static TRADE_ID: AtomicU64 = AtomicU64::new(1);
pub static USER_ID: AtomicU64 = AtomicU64::new(1);

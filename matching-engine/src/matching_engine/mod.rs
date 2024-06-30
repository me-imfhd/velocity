#![allow(non_camel_case_types)]
use std::{ cell::Cell, str::FromStr, sync::atomic::{ AtomicU64, Ordering } };
use engine::{ Exchange, RecievedOrder };
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use scylla::{ transport::errors::QueryError, FromRow, SerializeRow, Session };
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
    pub fn from_str(asset_to_match: &str) -> Option<Self> {
        for asset in Asset::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Some(asset);
            }
        }
        None
    }
}
pub type Symbol = String;
pub type Id = u64;
pub type OrderId = i64;
pub type Quantity = Decimal;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Serialize, Deserialize, EnumStringify)]
pub enum RegisteredSymbols {
    SOL_USDT,
    BTC_USDT,
    ETH_USDT,
}
#[derive(Debug, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaOrder {
    pub id: OrderId,
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
impl RecievedOrder {
    fn to_scylla_order(&self,order_id: OrderId) -> ScyllaOrder {
        ScyllaOrder {
            id: order_id,
            timestamp: self.timestamp as i64,
            user_id: self.user_id as i64,
            symbol: self.symbol.to_string(),
            filled_quantity: self.filled_quantity.to_string(),
            price: self.price.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            order_type: self.order_type.to_string(),
        }
    }
}
pub async fn new_order(session: &Session,order_id: OrderId, order: RecievedOrder) -> Result<(), QueryError> {
    let s =
        r#"
        INSERT INTO keyspace_1.order_table (
            id,
            user_id,
            symbol,
            price,
            initial_quantity,
            filled_quantity, 
            order_type,
            order_side,
            order_status,
            timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    "#;
    let order = order.to_scylla_order(order_id);
    session.query(s, order).await?;
    Ok(())
}

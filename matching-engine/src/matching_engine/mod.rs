#![allow(non_camel_case_types)]
use std::{
    cell::Cell,
    collections::HashMap,
    str::FromStr,
    sync::{ atomic::{ AtomicU64, Ordering }, Arc, Mutex },
};
use enum_stringify::EnumStringify;
use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use scylla::{ batch::Batch, transport::errors::QueryError, FromRow, SerializeRow, Session };
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use strum_macros::{ EnumIter, EnumString };

use crate::{ handle_order_request::CancelOrder, PersistCancel, PersistCancelAll };
pub mod orderbook;
pub mod engine;
pub mod error;
pub mod user;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Id,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Users {
    pub users: HashMap<Id, User>,
}

pub static USERS: Lazy<Mutex<Users>> = Lazy::new(|| {
    Mutex::new(Users {
        users: HashMap::new(),
    })
});

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
pub type Price = Decimal;
pub type Symbol = String;
pub type Id = u64;
pub type OrderId = u64;
pub type TradeId = u64;
pub type Quantity = Decimal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, EnumStringify)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumStringify)]
pub enum OrderSide {
    Bid,
    Ask,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Serialize, Deserialize, EnumStringify)]
pub enum RegisteredSymbols {
    SOL_USDT,
    BTC_USDT,
    ETH_USDT,
}
#[derive(Debug, Clone, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaUser {
    pub id: i64,
    pub balance: HashMap<String, String>,
    pub locked_balance: HashMap<String, String>,
}
#[derive(Debug, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: String,
    pub price: String,
    pub initial_quantity: String,
    pub filled_quantity: String,
    pub quote_quantity: String,
    pub filled_quote_quantity: String,
    pub order_type: String,
    pub order_side: String,
    pub order_status: String,
    pub timestamp: i64,
}
#[derive(Debug, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct ScyllaCancelOrder {
    pub id: i64,
    pub user_id: i64,
    pub order_side: String,
    pub symbol: String,
    pub price: String,
    pub timestamp: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecievedOrder {
    pub id: i64,
    pub user_id: i64,
    pub symbol: Symbol,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub quote_quantity: Quantity,
    pub filled_quote_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, EnumStringify)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Cancelled,
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

impl RecievedOrder {
    fn to_scylla_order(&self) -> ScyllaOrder {
        ScyllaOrder {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            filled_quantity: self.filled_quantity.to_string(),
            filled_quote_quantity: self.filled_quote_quantity.to_string(),
            quote_quantity: self.quote_quantity.to_string(),
            price: self.price.to_string(),
            initial_quantity: self.initial_quantity.to_string(),
            order_side: self.order_side.to_string(),
            order_status: self.order_status.to_string(),
            order_type: self.order_type.to_string(),
        }
    }
}
pub async fn persist_order_cancel_all(session: &Session, cancel_order: PersistCancelAll) {
    let new_cance_order =
        r#"
        INSERT INTO keyspace_1.cancel_order_table (
            id,
            user_id,
            order_side,
            symbol,
            price,
            timestamp
        ) VALUES (?, ?, ?, ?, ?, ?);
        "#;
    let unlock_balance =
        r#"
        UPDATE keyspace_1.user_table 
        SET
            locked_balance = ?
        WHERE id = ?;
        "#;
    let update_order_status =
        r#"
        UPDATE keyspace_1.order_table 
        SET
            order_status = ?
        WHERE id = ? AND symbol = ?;
        "#;
    for data in cancel_order.data {
        session
            .query(new_cance_order, (
                data.id as i64,
                cancel_order.user_id as i64,
                data.order_side.to_string(),
                cancel_order.symbol.clone(),
                data.price.to_string(),
                cancel_order.timestamp,
            )).await
            .unwrap();
        session
            .query(update_order_status, (
                OrderStatus::Cancelled.to_string(),
                data.id as i64,
                cancel_order.symbol.clone(),
            )).await
            .unwrap();
    }
    session
        .query(unlock_balance, (cancel_order.locked_balances, cancel_order.user_id as i64)).await
        .unwrap();
}
pub async fn persist_order_cancel(session: &Session, cancel_order: PersistCancel) {
    let new_cance_order =
        r#"
        INSERT INTO keyspace_1.cancel_order_table (
            id,
            user_id,
            order_side,
            symbol,
            price,
            timestamp
        ) VALUES (?, ?, ?, ?, ?, ?);
        "#;
    let unlock_balance =
        r#"
        UPDATE keyspace_1.user_table 
        SET
            locked_balance[?] = ?
        WHERE id = ?;
        "#;
    let update_order_status =
        r#"
        UPDATE keyspace_1.order_table 
        SET
            order_status = ?
        WHERE id = ? AND symbol = ?;
        "#;
    let mut batch: Batch = Default::default();
    batch.append_statement(new_cance_order);
    batch.append_statement(unlock_balance);
    batch.append_statement(update_order_status);
    let prepared_batch: Batch = session.prepare_batch(&batch).await.unwrap();
    session
        .batch(&prepared_batch, (
            (
                cancel_order.id as i64,
                cancel_order.user_id as i64,
                cancel_order.order_side.to_string(),
                cancel_order.symbol,
                cancel_order.price.to_string(),
                cancel_order.timestamp,
            ),
            (
                cancel_order.asset.to_string(),
                cancel_order.updated_locked_balance.to_string(),
                cancel_order.user_id as i64,
            ),
            (OrderStatus::Cancelled.to_string(), cancel_order.id as i64),
        )).await
        .unwrap();
}
pub async fn new_order(
    session: &Session,
    order: RecievedOrder,
    locked_balance: Quantity,
    lock_asset: Asset
) {
    let new_order =
        r#"
        INSERT INTO keyspace_1.order_table (
            id,
            user_id,
            symbol,
            price,
            initial_quantity,
            filled_quantity, 
            quote_quantity,
            filled_quote_quantity,
            order_type,
            order_side,
            order_status,
            timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    "#;
    let lock_balance =
        r#"
        UPDATE keyspace_1.user_table 
        SET
            locked_balance[?] = ?
        WHERE id = ?;
    "#;
    let mut batch: Batch = Default::default();
    batch.append_statement(new_order);
    batch.append_statement(lock_balance);
    let prepared_batch: Batch = session.prepare_batch(&batch).await.unwrap();
    let order_value = order.to_scylla_order();
    let user_value = (lock_asset.to_string(), locked_balance.to_string(), order.user_id as i64);
    session.batch(&prepared_batch, (order_value, user_value)).await.unwrap();
}
impl ScyllaOrder {
    fn from_scylla_order(&self) -> RecievedOrder {
        RecievedOrder {
            id: self.id,
            timestamp: self.timestamp,
            user_id: self.user_id,
            symbol: self.symbol.to_string(),
            filled_quote_quantity: Decimal::from_str(&self.filled_quote_quantity).unwrap(),
            quote_quantity: Decimal::from_str(&self.quote_quantity).unwrap(),
            filled_quantity: Decimal::from_str(&self.filled_quantity).unwrap(),
            price: Decimal::from_str(&self.price).unwrap(),
            initial_quantity: Decimal::from_str(&self.initial_quantity).unwrap(),
            order_side: OrderSide::from_str(&self.order_side).unwrap(),
            order_status: OrderStatus::from_str(&self.order_status).unwrap(),
            order_type: OrderType::from_str(&self.order_type).unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Filler {
    trade_id: Id,
    exchange: Exchange,
    quantity: Quantity,
    exchange_price: Price,
    is_buyer_maker: bool,
    post_users: PostUsers,
    order_status: OrderStatus,
    client_order_status: OrderStatus,
    order_id: OrderId,
    client_order_id: OrderId,
    timestamp: u128,
}
#[derive(Debug, Serialize)]
pub struct PostUsers {
    pub user: User,
    pub client: User,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: String,
}
#[derive(Debug, Serialize)]
pub enum SymbolError {
    InvalidSymbol,
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
    pub fn from_symbol(symbol: Symbol) -> Result<Exchange, SymbolError> {
        let symbols: Vec<&str> = symbol.split("_").collect();
        let base_str = symbols.get(0).ok_or(SymbolError::InvalidSymbol)?;
        let quote_str = symbols.get(1).ok_or(SymbolError::InvalidSymbol)?;
        let base = Asset::from_str(&base_str).ok_or(SymbolError::InvalidSymbol)?;
        let quote = Asset::from_str(&quote_str).ok_or(SymbolError::InvalidSymbol)?;
        let exchange = Exchange::new(base, quote);
        Ok(exchange)
    }
}

#[derive(Serialize, Deserialize)]
pub struct OrderUpdate {
    order_id: u64,
    client_order_id: u64,
    trade_id: u64,
    user_id: u64,
    trade_timestamp: u128,
    order_side: OrderSide,
    order_status: OrderStatus,
    symbol: String,
    price: Decimal,
    executed_quantity: Decimal,
    executed_quote_quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    id: Id,
    quantity: Quantity,
    quote_quantity: Quantity,
    is_buyer_maker: bool,
    timestamp: u128,
    price: Price,
}

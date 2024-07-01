#![allow(non_camel_case_types)]
use std::{ cell::Cell, str::FromStr, sync::atomic::{ AtomicU64, Ordering } };
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
pub type Price = Decimal;
pub type Symbol = String;
pub type Id = u64;
pub type OrderId = i64;
pub type Quantity = Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, EnumStringify)]
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
#[derive(Debug, Clone, Serialize, Deserialize, EnumStringify)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecievedOrder {
    #[serde(skip)]
    pub id: OrderId,
    pub user_id: Id,
    pub symbol: Symbol,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: u64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveOrder {
    pub id: OrderId,
    pub recieved_order: RecievedOrder,
}
#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, EnumStringify)]
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

impl RecievedOrder {
    fn to_scylla_order(&self, order_id: OrderId) -> ScyllaOrder {
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
pub async fn new_order(
    session: &Session,
    order_id: OrderId,
    order: RecievedOrder
) -> Result<(), QueryError> {
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
impl SerializedOrder {
    fn from_scylla_order(&self) -> ReplayOrder {
        ReplayOrder {
            id: self.id,
            timestamp: self.timestamp as u64,
            user_id: self.user_id as u64,
            symbol: self.symbol.to_string(),
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
pub struct QueueTrade {
    trade_id: Id,
    user_id_1: Id,
    user_id_2: Id,
    exchange: Exchange,
    base_quantity: Quantity,
    price: Price,
    is_market_maker: bool,
    order_status_1: OrderStatus,
    order_status_2: OrderStatus,
    order_id_1: OrderId,
    order_id_2: OrderId,
    timestamp: u128,
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

#[derive(Debug, Deserialize, Serialize, SerializeRow, FromRow)]
pub struct SerializedOrder {
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
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplayOrder {
    pub id: OrderId,
    pub user_id: u64,
    pub symbol: String,
    pub price: Price,
    pub initial_quantity: Quantity,
    pub filled_quantity: Quantity,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub order_status: OrderStatus,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct OrderUpdate {
    order_id: i64,
    client_order_id: i64,
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
    is_market_maker: bool,
    timestamp: u128,
    price: Price,
}

use std::{ collections::HashMap, error::Error };
use enum_stringify::EnumStringify;
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub type Id = i64;
pub type OrderId = i64;
pub type Symbol = String;
pub type Quantity = Decimal;
pub type Price = Decimal;

#[derive(Debug, Serialize)]
pub enum SymbolError {
    InvalidSymbol,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: Symbol,
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

#[derive(Debug, Deserialize, Serialize, EnumStringify, EnumIter)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Cancelled
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: Id,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: Id,
    pub symbol: Symbol,
    pub quantity: Quantity,
    pub quote_quantity: Quantity,
    pub is_buyer_maker: bool,
    pub price: Price,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ticker {
    pub symbol: Symbol,
    pub base_volume: Quantity,
    pub quote_volume: Quantity,
    pub price_change: Quantity,
    pub price_change_percent: Quantity,
    pub high_price: Quantity,
    pub low_price: Quantity,
    pub last_price: Quantity,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Market {
    pub symbol: Symbol,
    pub base: Asset,
    pub quote: Asset,
    pub max_price: Quantity,
    pub min_price: Quantity,
    pub tick_size: Quantity,
    pub max_quantity: Quantity,
    pub min_quantity: Quantity,
    pub step_size: Quantity,
}

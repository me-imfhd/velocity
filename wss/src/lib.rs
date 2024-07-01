#![allow(non_camel_case_types)]
use std::sync::{ Arc, Mutex };

use enum_stringify::EnumStringify;
use manager::UserManager;
use once_cell::sync::Lazy;
use redis::Connection;
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use serde_json::from_str;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::{ net::TcpStream, runtime::{ Builder, Runtime } };
use tokio_tungstenite::{ accept_async, WebSocketStream };
pub mod manager;
pub async fn handshake(
    raw_stream: TcpStream
) -> Result<WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Error> {
    let result = accept_async(raw_stream).await;
    match result {
        Ok(ws_stream) => Ok(ws_stream),
        Err(err) => Err(err),
    }
}

#[derive(Deserialize)]
pub struct Payload {
    pub user_id: Option<u64>,
    pub method: Method,
    pub event: Event,
    pub symbol: RegisteredSymbols,
}
#[derive(Deserialize, PartialEq, Eq, Hash, Clone, EnumIter, EnumStringify)]
pub enum Event {
    ORDER_UPDATE,
    TRADE,
    TICKER,
    DEPTH,
}
#[derive(Deserialize)]
pub enum Method {
    SUBSCRIBE,
    UNSUBSCRIBE,
}
#[derive(Deserialize, PartialEq, Eq, Hash, EnumIter, EnumStringify)]
pub enum RegisteredSymbols {
    SOL_USDT,
    BTC_USDT,
    ETH_USDT,
}
impl RegisteredSymbols {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in RegisteredSymbols::iter() {
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
pub type OrderId = i64;
pub type Quantity = Decimal;
pub type Price = Decimal;
#[derive(Deserialize)]
pub struct QueueTrade {
    pub trade_id: Id,
    pub user_id_1: Id,
    pub user_id_2: Id,
    pub exchange: Exchange,
    pub base_quantity: Quantity,
    pub price: Price,
    pub is_market_maker: bool,
    pub order_status_1: OrderStatus,
    pub order_status_2: OrderStatus,
    pub order_id_1: Id,
    pub order_id_2: Id,
}
#[derive(Debug, Clone, Deserialize, EnumIter, EnumStringify)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Deserialize, EnumStringify)]
pub enum Asset {
    USDT,
    BTC,
    SOL,
    ETH,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: String,
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
#[derive(Debug)]
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
pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_current_thread().thread_name("tokio").enable_all().build().unwrap()
});

pub fn handle_brodcasting_trades(
    manager: Arc<Mutex<UserManager>>,
    mut con: Connection
) -> impl FnMut() {
    move || {
        let mut pub_sub = con.as_pubsub();
        for symbol in RegisteredSymbols::iter() {
            let symbol = symbol.to_string();
            if let Err(err) = pub_sub.subscribe(format!("trades:{}", symbol)) {
                println!("Could not subscribe to trade pubsub for symbol :{}, {}", symbol, err);
            }
        }
        loop {
            if let Ok(msg) = pub_sub.get_message() {
                if let Ok(trade) = msg.get_payload::<String>() {
                    let mut manager = manager.lock().unwrap();
                    let symbol_str = msg.get_channel_name().split(":").last().unwrap();
                    let symbol = RegisteredSymbols::from_str(symbol_str).unwrap();
                    TOKIO_RUNTIME.block_on(manager.brodcast_trade(symbol, trade));
                }
            }
        }
    }
}
pub fn handle_brodcasting_ticker(
    manager: Arc<Mutex<UserManager>>,
    mut con: Connection
) -> impl FnMut() {
    move || {
        let mut pub_sub = con.as_pubsub();
        for symbol in RegisteredSymbols::iter() {
            let symbol = symbol.to_string();
            if let Err(err) = pub_sub.subscribe(format!("ticker:{}", symbol)) {
                println!("Could not subscribe to ticker pubsub symbol :{}, {}", symbol, err);
            }
        }
        loop {
            if let Ok(msg) = pub_sub.get_message() {
                if let Ok(ticker) = msg.get_payload::<String>() {
                    let mut manager = manager.lock().unwrap();
                    let symbol_str = msg.get_channel_name().split(":").last().unwrap();
                    let symbol = RegisteredSymbols::from_str(symbol_str).unwrap();
                    TOKIO_RUNTIME.block_on(manager.brodcast_ticker(symbol, ticker));
                }
            }
        }
    }
}
pub fn handle_brodcasting_depth(
    manager: Arc<Mutex<UserManager>>,
    mut con: Connection
) -> impl FnMut() {
    move || {
        let mut pub_sub = con.as_pubsub();
        for symbol in RegisteredSymbols::iter() {
            let symbol = symbol.to_string();
            if let Err(err) = pub_sub.subscribe(format!("depth:{}", symbol)) {
                println!("Could not subscribe to  depth pubsub symbol :{}, {}", symbol, err);
            }
        }
        loop {
            if let Ok(msg) = pub_sub.get_message() {
                if let Ok(depth) = msg.get_payload::<String>() {
                    let mut manager = manager.lock().unwrap();
                    let symbol_str = msg.get_channel_name().split(":").last().unwrap();
                    let symbol = RegisteredSymbols::from_str(symbol_str).unwrap();
                    TOKIO_RUNTIME.block_on(manager.brodcast_depth(symbol, depth));
                }
            }
        }
    }
}
pub fn handle_order_update_stream(
    manager: Arc<Mutex<UserManager>>,
    mut con: Connection
) -> impl FnMut() {
    move || {
        let mut pub_sub = con.as_pubsub();
        for symbol in RegisteredSymbols::iter() {
            let symbol = symbol.to_string();
            if let Err(err) = pub_sub.subscribe(format!("order_update:{}", symbol)) {
                println!("Could not subscribe to  order_update pubsub symbol :{}, {}", symbol, err);
            }
        }
        loop {
            if let Ok(msg) = pub_sub.get_message() {
                if let Ok(order_update_string) = msg.get_payload::<String>() {
                    if let Ok(order_update) = from_str::<OrderUpdate>(&order_update_string) {
                        TOKIO_RUNTIME.block_on(
                            manager.lock().unwrap().send_order_update(order_update.user_id, &order_update_string)
                        );
                    }
                }
            }
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderUpdate {
    order_id: i64,
    client_order_id: i64,
    trade_id: u64,
    user_id: u64,
    trade_timestamp: u128,
    order_side: String,
    order_status: String,
    symbol: String,
    price: Decimal,
    executed_quantity: Decimal,
    executed_quote_quantity: Decimal,
}

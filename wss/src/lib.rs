#![allow(non_camel_case_types)]
use enum_stringify::EnumStringify;
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::net::TcpStream;
use tokio_tungstenite::{ accept_async, WebSocketStream };
pub mod trade_manager;
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
    pub method: Method,
    pub event: Event,
    pub symbol: RegisteredSymbols,
}
#[derive(Deserialize, PartialEq, Eq, Clone)]
pub enum Event {
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

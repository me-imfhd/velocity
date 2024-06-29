use std::collections::HashMap;

use futures_util::{ stream::SplitSink, SinkExt };
use tokio::net::TcpStream;
use tokio_tungstenite::{ tungstenite::Message, WebSocketStream };

use crate::RegisteredSymbols;

pub struct TradeManager {
    pub users: HashMap<String, Info>,
}
pub struct Info {
    pub transmitter: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub symbols_subscribed: Vec<RegisteredSymbols>,
}

impl TradeManager {
    pub fn new() -> TradeManager {
        TradeManager {
            users: HashMap::new(),
        }
    }
    pub fn new_user(
        &mut self,
        user_addr: String,
        tx: SplitSink<WebSocketStream<TcpStream>, Message>
    ) {
        self.users.insert(user_addr, Info {
            transmitter: tx,
            symbols_subscribed: Vec::new(),
        });
    }
    pub fn remove_user(&mut self, user_addr: String) {
        self.users.remove(&user_addr);
    }
    pub fn subscribe(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        self.users.get_mut(&user_addr).unwrap().symbols_subscribed.push(symbol);
    }
    pub fn unsubscribe(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        self.users
            .get_mut(&user_addr)
            .unwrap()
            .symbols_subscribed.retain(|syb| syb != &symbol);
    }
    pub async fn brodcast_trade(&mut self, symbol: RegisteredSymbols, trade: String) {
        for user in self.users.values_mut() {
            if user.symbols_subscribed.contains(&symbol) {
                let message = Message::text(trade.clone());
                if let Err(err) = user.transmitter.send(message).await {
                    eprintln!("Could not send trade, error occured: {}", err);
                }
            }
        }
    }
}

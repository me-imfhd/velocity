use std::collections::HashMap;

use futures_util::{ stream::SplitSink, SinkExt };
use tokio::net::TcpStream;
use tokio_tungstenite::{ tungstenite::Message, WebSocketStream };

use crate::RegisteredSymbols;
pub struct UserManager {
    pub users: HashMap<String, UserInfo>,
}
pub struct UserInfo {
    pub user_id: Option<u64>,
    pub transmitter: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub trade_subscriptions: Vec<RegisteredSymbols>,
    pub ticker_subscriptions: Vec<RegisteredSymbols>,
    pub depth_subscriptions: Vec<RegisteredSymbols>,
}

impl UserManager {
    pub fn new() -> UserManager {
        UserManager {
            users: HashMap::new(),
        }
    }
    pub fn new_user(
        &mut self,
        user_addr: String,
        tx: SplitSink<WebSocketStream<TcpStream>, Message>
    ) {
        self.users.insert(user_addr, UserInfo {
            transmitter: tx,
            user_id: None,
            depth_subscriptions: Vec::new(),
            ticker_subscriptions: Vec::new(),
            trade_subscriptions: Vec::new(),
        });
    }
    // Will be used to send order_update stream
    pub fn assign_user_id(&mut self, user_addr: String, user_id: Option<u64>) {
        if let Some(user_info) = self.users.get_mut(&user_addr) {
            if let Some(user_id) = user_id {
                user_info.user_id = Some(user_id);
                println!("Assigned user_id for the ws connection")
            }
        }
    }
    pub fn dissociate_user_id(&mut self, user_addr: String) {
        if let Some(user_info) = self.users.get_mut(&user_addr) {
            user_info.user_id = None;
        }
    }
    pub async fn send_order_update(&mut self, user_id: u64, order_update: &str) {
        let user = self.users.values_mut().find(|u| {
            if let Some(uid) = u.user_id {
                return uid == user_id;
            }
            false
        });
        if let Some(user) = user {
            let _ = user.transmitter.send(Message::text(order_update)).await;
        }
    }
    pub fn remove_user(&mut self, user_addr: String) {
        self.users.remove(&user_addr);
    }
}

impl UserManager {
    pub fn subscribe_trades(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        if let Some(user) = self.users.get_mut(&user_addr) {
            user.trade_subscriptions.push(symbol);
            println!("Subscribed to trades")
        }
    }
    pub fn unsubscribe_trades(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        if let Some(user) = self.users.get_mut(&user_addr) {
            user.trade_subscriptions.retain(|syb| syb != &symbol);
            println!("Unsubscribed to trades")
        }
    }
    pub async fn brodcast_trade(&mut self, symbol: RegisteredSymbols, trade: String) {
        for user in self.users.values_mut() {
            if user.trade_subscriptions.contains(&symbol) {
                let message = Message::text(trade.clone());
                if let Err(err) = user.transmitter.send(message).await {
                    eprintln!("Could not send trade, error occured: {}", err);
                }
            }
        }
    }
}

impl UserManager {
    pub fn subscribe_ticker(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        if let Some(user) = self.users.get_mut(&user_addr) {
            user.ticker_subscriptions.push(symbol);
            println!("Subscribed to ticker")
        }
    }
    pub fn unsubscribe_ticker(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        if let Some(user) = self.users.get_mut(&user_addr) {
            user.ticker_subscriptions.retain(|syb| syb != &symbol);
            println!("Unsubscribed to ticker")
        }
    }
    pub async fn brodcast_ticker(&mut self, symbol: RegisteredSymbols, ticker: String) {
        for user in self.users.values_mut() {
            if user.ticker_subscriptions.contains(&symbol) {
                let message = Message::text(ticker.clone());
                if let Err(err) = user.transmitter.send(message).await {
                    eprintln!("Could not send ticker, error occured: {}", err);
                }
            }
        }
    }
}
impl UserManager {
    pub fn subscribe_depth(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        if let Some(user) = self.users.get_mut(&user_addr) {
            user.depth_subscriptions.push(symbol);
            println!("Subscribed to depth")
        }
    }
    pub fn unsubscribe_depth(&mut self, user_addr: String, symbol: RegisteredSymbols) {
        if let Some(user) = self.users.get_mut(&user_addr) {
            user.depth_subscriptions.retain(|syb| syb != &symbol);
            println!("Unsubscribed to depth")
        }
    }
    pub async fn brodcast_depth(&mut self, symbol: RegisteredSymbols, depth: String) {
        for user in self.users.values_mut() {
            if user.depth_subscriptions.contains(&symbol) {
                let message = Message::text(depth.clone());
                if let Err(err) = user.transmitter.send(message).await {
                    eprintln!("Could not send depth, error occured: {}", err);
                }
            }
        }
    }
}

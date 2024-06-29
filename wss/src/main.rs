use std::sync::{ Arc, Mutex };

use futures_util::StreamExt;
use strum::IntoEnumIterator;
use tokio::net::{ TcpListener, TcpStream };
use tokio_tungstenite::{ tungstenite::protocol::Message, WebSocketStream };
use wss::{ handshake, trade_manager::TradeManager, Event, Method, Payload, RegisteredSymbols };

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080".to_string();
    let client = redis::Client::open("redis://127.0.0.1/").expect("Could not create client");
    let mut con = client.get_connection().expect("Could not connect");

    let trades_manager = Arc::new(Mutex::new(TradeManager::new()));
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("Listening on: {}", addr);
    let stream_trade_manager = trades_manager.clone();
    tokio::spawn(async move {
        while let Ok((stream, user_addr)) = listener.accept().await {
            let trades_manager = stream_trade_manager.clone();
            let new_ws_connection = async move {
                if let Ok(ws_stream) = handshake(stream).await {
                    handle_stream(ws_stream, trades_manager, user_addr.to_string()).await;
                }
            };
            tokio::spawn(new_ws_connection);
        }
    });
    let mut pubsub = con.as_pubsub();
    for symbol in RegisteredSymbols::iter() {
        let symbol = symbol.to_string();
        if let Err(err) = pubsub.subscribe(format!("trades:{}", symbol)) {
            println!("Cannot subscribe to symbol :{}, {}", symbol, err);
        }
    }
    loop {
        let manager = trades_manager.clone();
        if let Ok(msg) = pubsub.get_message() {
            if let Ok(trade) = msg.get_payload::<String>() {
                let mut manager = manager.lock().unwrap();
                let symbol_str = msg.get_channel_name().split(":").last().unwrap();
                let symbol = RegisteredSymbols::from_str(symbol_str).unwrap();
                manager.brodcast_trade(symbol, trade).await;
            }
        }
    }
}

async fn handle_stream(
    ws_stream: WebSocketStream<TcpStream>,
    trade_manager: Arc<Mutex<TradeManager>>,
    user_addr: String
) {
    let (write, mut read) = ws_stream.split();
    {
        let mut manager = trade_manager.lock().unwrap();
        manager.new_user(user_addr.to_string(), write);
        println!("New WebSocket connection established and user registered from: {}", user_addr);
    }
    let handle_client_incoming = async move {
        while let Some(Ok(msg)) = read.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(payload) = serde_json::from_str::<Payload>(&text) {
                        handle_payload(payload, user_addr.clone(), trade_manager.clone());
                    }
                }
                Message::Close(_) => {
                    let mut manager = trade_manager.lock().unwrap();
                    manager.remove_user(user_addr.to_string());
                    println!("WebSocket connection closed from address: {}", user_addr);
                }
                _ => {}
            }
        }
    };
    tokio::spawn(handle_client_incoming);
}

fn handle_payload(payload: Payload, user_addr: String, trade_manager: Arc<Mutex<TradeManager>>) {
    match payload.event {
        Event::TRADE => {
            let mut trade_manager = trade_manager.lock().unwrap();
            match payload.method {
                Method::SUBSCRIBE => {
                    trade_manager.subscribe(user_addr, payload.symbol);
                }
                Method::UNSUBSCRIBE => {
                    trade_manager.unsubscribe(user_addr, payload.symbol);
                }
            }
        }
        Event::TICKER => todo!(),
        Event::DEPTH => todo!(),
    }
}

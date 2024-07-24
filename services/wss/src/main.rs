use std::{ sync::{ Arc, Mutex }, thread };

use futures_util::StreamExt;
use tokio::net::{ TcpListener, TcpStream };
use tokio_tungstenite::{ tungstenite::protocol::Message, WebSocketStream };
use wss::{
    handle_brodcasting_depth,
    handle_brodcasting_ticker,
    handle_brodcasting_trades,
    handle_order_update_stream,
    handshake,
    manager::UserManager,
    Event,
    Method,
    Payload,
};
fn main() {
    let addr = "127.0.0.1:9000".to_string();
    let client = redis::Client::open("redis://127.0.0.1/").expect("Could not create client");
    let trade_con = client.get_connection().expect("Could not connect");
    let ticker_con = client.get_connection().expect("Could not connect");
    let depth_con = client.get_connection().expect("Could not connect");
    let order_update_con = client.get_connection().expect("Could not connect");

    let user_manager = Arc::new(Mutex::new(UserManager::new()));

    let trade_user_manager = user_manager.clone();
    let ticker_user_manager = user_manager.clone();
    let depth_user_manager = user_manager.clone();
    let order_update_user_manager = user_manager.clone();
    thread::spawn(handle_brodcasting_trades(trade_user_manager, trade_con));
    thread::spawn(handle_brodcasting_ticker(ticker_user_manager, ticker_con));
    thread::spawn(handle_brodcasting_depth(depth_user_manager, depth_con));
    thread::spawn(handle_order_update_stream(order_update_user_manager, order_update_con));

    let ws_server = async move {
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        println!("Listening on: {}", addr);
        while let Ok((stream, user_addr)) = listener.accept().await {
            let user_manager = user_manager.clone();
            let new_ws_connection = async move {
                if let Ok(ws_stream) = handshake(stream).await {
                    handle_stream(ws_stream, user_manager, user_addr.to_string()).await;
                }
            };
            tokio::spawn(new_ws_connection);
        }
    };
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(ws_server);
}
async fn handle_stream(
    ws_stream: WebSocketStream<TcpStream>,
    user_manager: Arc<Mutex<UserManager>>,
    user_addr: String
) {
    let (write, mut read) = ws_stream.split();
    {
        let mut manager = user_manager.lock().unwrap();
        manager.new_user(user_addr.to_string(), write);
        println!("New WebSocket connection established and user registered from: {}", user_addr);
    }
    let handle_client_incoming = async move {
        while let Some(Ok(msg)) = read.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(payload) = serde_json::from_str::<Payload>(&text) {
                        handle_payload(payload, user_addr.clone(), user_manager.clone());
                    }
                }
                Message::Close(_) => {
                    let mut manager = user_manager.lock().unwrap();
                    manager.remove_user(user_addr.to_string());
                    println!("WebSocket connection closed from address: {}", user_addr);
                }
                _ => {}
            }
        }
    };
    tokio::spawn(handle_client_incoming);
}

fn handle_payload(payload: Payload, user_addr: String, user_manager: Arc<Mutex<UserManager>>) {
    let mut user_manager = user_manager.lock().unwrap();
    match payload.event {
        Event::TRADE => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.subscribe_trades(user_addr, payload.symbol);
                }
                Method::UNSUBSCRIBE => {
                    user_manager.unsubscribe_trades(user_addr, payload.symbol);
                }
            }
        }
        Event::TICKER => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.subscribe_ticker(user_addr, payload.symbol);
                }
                Method::UNSUBSCRIBE => {
                    user_manager.unsubscribe_ticker(user_addr, payload.symbol);
                }
            }
        }
        Event::DEPTH => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.subscribe_depth(user_addr, payload.symbol);
                }
                Method::UNSUBSCRIBE => {
                    user_manager.unsubscribe_depth(user_addr, payload.symbol);
                }
            }
        }
        // This needs authentication layer
        Event::ORDER_UPDATE => {
            match payload.method {
                Method::SUBSCRIBE => {
                    user_manager.assign_user_id(user_addr, payload.user_id);
                }
                Method::UNSUBSCRIBE => {
                    user_manager.dissociate_user_id(user_addr);
                }
            }
        }
    }
}

#![allow(unused)]

use std::{
    collections::HashMap,
    fmt::format,
    io::Write,
    sync::{ mpsc::Sender, Arc, Mutex },
    thread,
    time::Instant,
};

use actix_web::web;
use engine::MatchingEngine;
use handle_order_request::{ CancelOrder, EngineRequests };
use handle_user_requests::UserRequests;
use matching_engine::*;
use once_cell::sync::Lazy;
use orderbook::Orderbook;
use redis::{ Connection, Value };
use rust_decimal::Decimal;
use scylla::{ Session, SessionBuilder };
use serde::{ Deserialize, Serialize };
use serde_json::{ from_str, to_string };
use tokio::{
    runtime::{ Builder, Runtime },
    sync::mpsc::{ self, UnboundedReceiver, UnboundedSender },
};
pub mod matching_engine;
pub mod handle_order_request;
pub mod handle_user_requests;
pub struct AppState {
    pub matching_engine: Mutex<MatchingEngine>,
}
pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_current_thread().thread_name("tokio").enable_all().build().unwrap()
});
pub fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create client.");
    let mut connection = client.get_connection().expect("Could not connect to the client");
    return connection;
}

pub fn process_user_request() -> impl Fn() {
    || {
        let mut con = connect_redis("redis://127.0.0.1:6379");
        loop {
            let result = redis::cmd("RPOP").arg("queues:user").query::<String>(&mut con);
            if let Ok(req_str) = result {
                if let Ok(request) = from_str::<UserRequests>(&req_str) {
                    let mut users = USERS.lock().unwrap();
                    match request {
                        UserRequests::NewUser(u) => UserRequests::new_user(&mut users, u, &mut con),
                        UserRequests::Deposit(u) => UserRequests::deposit(&mut users, u, &mut con),
                        UserRequests::Withdraw(u) =>
                            UserRequests::withdraw(&mut users, u, &mut con),
                        UserRequests::GetUserBalances(u) =>
                            UserRequests::get_user_balances(&mut users, u, &mut con),
                    }
                }
            }
        }
    }
}
pub fn process_order(mut orderbook: Orderbook) -> impl FnMut() {
    move || {
        let mut con = connect_redis("redis://127.0.0.1:6379");
        println!("OS Thread Created For {}", orderbook.exchange.symbol);
        let (tx, mut rx) = mpsc::unbounded_channel::<PersistOrderRequest>();
        thread::spawn(persist_requests(rx));
        loop {
            let start = Instant::now();
            let tx = tx.clone();
            let result = redis
                ::cmd("RPOP")
                .arg(format!("queues:{}", orderbook.exchange.symbol))
                .query::<String>(&mut con);
            if let Ok(request) = result {
                if let Ok(request) = from_str::<EngineRequests>(&request) {
                    match request {
                        EngineRequests::ExecuteOrder(recieved_order) =>
                            EngineRequests::execute_order(
                                start,
                                recieved_order,
                                &mut orderbook,
                                &mut con,
                                tx
                            ),
                        EngineRequests::CancelOrder(c_order) =>
                            EngineRequests::cancel_order(
                                start,
                                c_order,
                                &mut orderbook,
                                &mut con,
                                tx
                            ),
                        EngineRequests::CancelAll(c_all) =>
                            EngineRequests::cancel_all_order(
                                start,
                                c_all,
                                &mut orderbook,
                                &mut con,
                                tx
                            ),
                        EngineRequests::OpenOrders(o_orders) =>
                            EngineRequests::open_orders(start, o_orders, &mut orderbook, &mut con),
                        EngineRequests::OpenOrder(o_order) =>
                            EngineRequests::open_order(start, o_order, &mut orderbook, &mut con),
                    }
                }
            }
        }
    }
}

pub fn persist_requests(mut rx: UnboundedReceiver<PersistOrderRequest>) -> impl FnMut() {
    move || {
        let session = TOKIO_RUNTIME.block_on(
            SessionBuilder::new().known_node("127.0.0.1:9042").build()
        ).unwrap();
        loop {
            if let Ok(order) = rx.try_recv() {
                let start = Instant::now();
                match order {
                    PersistOrderRequest::Save(s_order) => {
                        TOKIO_RUNTIME.block_on(
                            new_order(
                                &session,
                                s_order.recieved_order,
                                s_order.locked_balance,
                                s_order.asset
                            )
                        );
                    }
                    PersistOrderRequest::Cancel(c_order) =>
                        TOKIO_RUNTIME.block_on(persist_order_cancel(&session, c_order)),
                    PersistOrderRequest::CancelAll(c_all) =>
                        TOKIO_RUNTIME.block_on(persist_order_cancel_all(&session, c_all)),
                }

                println!("\tPersisted order request in {}ms\n", start.elapsed().as_millis());
            }
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PersistOrderRequest {
    Save(SaveOrder),
    Cancel(PersistCancel),
    CancelAll(PersistCancelAll),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistCancel {
    pub id: OrderId,
    pub user_id: Id,
    pub symbol: Symbol,
    pub price: Price,
    pub order_side: OrderSide,
    pub asset: Asset,
    pub updated_locked_balance: Quantity,
    pub timestamp: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersistCancelAll {
    user_id: i64,
    symbol: Symbol,
    timestamp: i64,
    data: Vec<OrderCancelInfo>,
    locked_balances: HashMap<String, String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderCancelInfo {
    id: i64,
    order_side: OrderSide,
    price: Price,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveOrder {
    pub locked_balance: Quantity,
    pub asset: Asset,
    pub recieved_order: RecievedOrder,
}

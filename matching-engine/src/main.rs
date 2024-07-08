#![allow(unused)]
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;
use actix_web::web;
use matching_engine::connect_redis;
use matching_engine::matching_engine::engine::MatchingEngine;
use matching_engine::matching_engine::new_order;
use matching_engine::matching_engine::Exchange;
use matching_engine::matching_engine::RegisteredSymbols;
use matching_engine::process_order;
use matching_engine::process_user_request;
use matching_engine::AppState;
use matching_engine::TOKIO_RUNTIME;
use once_cell::sync::Lazy;
use scylla::SessionBuilder;
use strum::IntoEnumIterator;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn main() {
    let mut redis_connection = connect_redis("redis://127.0.0.1:6379");
    let session = TOKIO_RUNTIME.block_on(
        SessionBuilder::new().known_node("127.0.0.1:9042").build()
    ).unwrap();
    let session = Arc::new(session);
    let session_clone = session.clone();
    let mut matching_engine = MatchingEngine::init();
    // Block and recover orderbooks on restart
    TOKIO_RUNTIME.block_on(matching_engine.recover_all_orderbooks(&session, &mut redis_connection));
    // create user, deposit, withdraw
    thread::spawn(process_user_request());
    // Running registered orderbooks engines parallely
    RegisteredSymbols::iter().for_each(|symbol| {
        let exchange = Exchange::from_symbol(symbol.to_string()).unwrap();
        let orderbook = matching_engine.orderbooks.get(&exchange).unwrap();
        thread::spawn(process_order(orderbook.clone()));
    });
    loop {
    }
}

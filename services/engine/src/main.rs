#![allow(unused)]
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;
use actix_web::web;
use engine::connect_redis;
use engine::matching_engine::engine::MatchingEngine;
use engine::matching_engine::new_order;
use engine::matching_engine::Exchange;
use engine::matching_engine::RegisteredSymbols;
use engine::process_order;
use engine::process_user_request;
use engine::AppState;
use engine::TOKIO_RUNTIME;
use once_cell::sync::Lazy;
use scylla::SessionBuilder;
use strum::IntoEnumIterator;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn main() {
    let session = TOKIO_RUNTIME.block_on(
        SessionBuilder::new().known_node("127.0.0.1:9042").build()
    ).unwrap();
    let mut matching_engine = MatchingEngine::init();
    // Block and recover orderbooks on restart
    TOKIO_RUNTIME.block_on(matching_engine.recover_all_orderbooks(&session));
    // Running registered orderbooks engines parallely
    RegisteredSymbols::iter().for_each(|symbol| {
        let exchange = Exchange::from_symbol(symbol.to_string()).unwrap();
        let orderbook = matching_engine.orderbooks.get(&exchange).unwrap();
        thread::spawn(process_order(orderbook.clone()));
    });
    // process captial request parallely, like deposit withdrawl
    thread::spawn(process_user_request());
    loop {
    }
}

// We are using total main + 1 + (3 * num_of_registered_symbol) System threads

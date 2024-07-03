#![allow(unused)]

use std::{ io::Write, sync::{ mpsc::Sender, Arc, Mutex }, time::Instant };

use actix_web::web;
use engine::MatchingEngine;
use matching_engine::*;
use redis::Connection;
use serde_json::from_str;
use tokio::sync::mpsc::UnboundedSender;
pub mod routes;
pub mod matching_engine;
pub mod config;
pub mod app;
pub struct AppState {
    pub matching_engine: Mutex<MatchingEngine>,
}

pub fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create client.");
    let mut connection = client.get_connection().expect("Could not connect to the client");
    return connection;
}

pub fn process_order(
    symbol: String,
    app_state: web::Data<AppState>,
    tx: UnboundedSender<SaveOrder>
) -> impl Fn() {
    move || {
        let mut con = connect_redis("redis://127.0.0.1:6379");
        println!("OS Thread Created For {}", symbol);
        loop {
            let start = Instant::now();
            let result = redis
                ::cmd("RPOP")
                .arg(format!("queues:{}", symbol))
                .query::<String>(&mut con);
            match result {
                Ok(order_string) => {
                    if let Ok(recieved_order) = from_str::<RecievedOrder>(&order_string) {
                        let mut matching_engine = app_state.matching_engine.lock().unwrap();
                        let exchange = Exchange::from_symbol(
                            recieved_order.symbol.clone()
                        ).unwrap();
                        let result = matching_engine.users.validate_and_lock_balance(
                            recieved_order.order_side.clone(),
                            &exchange,
                            recieved_order.user_id,
                            recieved_order.price,
                            recieved_order.initial_quantity
                        );
                        match result {
                            Ok((asset, locked_balance)) => {
                                let order_id = matching_engine.increment_order_id(&exchange);
                                tx.send(SaveOrder {
                                    id: order_id,
                                    locked_balance,
                                    asset,
                                    recieved_order: recieved_order.clone(),
                                });
                                matching_engine.process_order(
                                    recieved_order,
                                    order_id,
                                    &exchange,
                                    &mut con
                                );
                                println!("Processed order in {} ms", start.elapsed().as_millis());
                            }
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
                Err(_) => {
                    // println!("Task queue empty, symbol: {}", symbol);
                }
            }
        }
    }
}

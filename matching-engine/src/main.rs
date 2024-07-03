#![allow(unused)]
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;
use actix_web::web;
use matching_engine::app::Application;
use matching_engine::config::get_config;
use matching_engine::connect_redis;
use matching_engine::matching_engine::engine::MatchingEngine;
use matching_engine::matching_engine::new_order;
use matching_engine::matching_engine::RegisteredSymbols;
use matching_engine::matching_engine::SaveOrder;
use matching_engine::process_order;
use matching_engine::AppState;
use once_cell::sync::Lazy;
use scylla::SessionBuilder;
use strum::IntoEnumIterator;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_current_thread().thread_name("tokio").enable_all().build().unwrap()
});

fn main() {
    dotenv::dotenv().ok();
    let mut redis_connection = connect_redis("redis://127.0.0.1:6379");
    let session = TOKIO_RUNTIME.block_on(
        SessionBuilder::new().known_node("127.0.0.1:9042").build()
    ).unwrap();
    let session = Arc::new(session);
    let session_clone = session.clone();
    let (tx, mut rx) = mpsc::unbounded_channel::<SaveOrder>();
    let mut matching_engine = MatchingEngine::init();
    // Block and recover orderbooks on restart
    TOKIO_RUNTIME.block_on(matching_engine.recover_all_orderbooks(&session, &mut redis_connection));
    let app_state = web::Data::new(AppState {
        matching_engine: Mutex::new(matching_engine),
    });
    // Running registered orderbooks engines parallely
    RegisteredSymbols::iter().for_each(|symbol| {
        let app_state = app_state.clone();
        thread::spawn(process_order(symbol.to_string(), app_state, tx.clone()));
    });
    // Saving orders via channels parallely
    thread::spawn(move || {
        loop {
            if let Ok(order) = rx.try_recv() {
                let start = Instant::now();
                TOKIO_RUNTIME.block_on(
                    new_order(
                        &session_clone,
                        order.id,
                        order.recieved_order,
                        order.locked_balance,
                        order.asset
                    )
                );
                println!("\tSaved order in {} ms\n", start.elapsed().as_millis());
            } else {
                // println!("rx empty");
            }
        }
    });
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async move {
        dotenv::dotenv().ok();
        let config = get_config().expect("Failed to read config");
        let application = Application::build(config, app_state).await.unwrap();
        application.run_until_stopped().await.unwrap();
    })
}

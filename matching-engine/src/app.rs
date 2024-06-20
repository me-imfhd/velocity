use std::{ any, collections::HashMap, net::TcpListener, sync::Mutex };
use rayon::prelude::*;

use actix_web::{ web::{ self, scope, Data }, App, HttpServer };
use redis::{ Commands, Connection };
use serde::{ Deserialize, Serialize };

use crate::{
    config::GlobalConfig,
    matching_engine::{ self, engine::MatchingEngine, orderbook::OrderSide, Id },
    routes::{ engine::{ add_new_market, get_asks, get_bids, get_quote }, health::health_check },
};

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    pub async fn build(config: GlobalConfig) -> Result<Self, std::io::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listner = TcpListener::bind(&address)?;
        let port = listner.local_addr().unwrap().port();
        let server = run(listner).await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct AppState {
    pub matching_engine: Mutex<MatchingEngine>,
    pub redis_connection: Mutex<Connection>,
}
async fn run(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let mut redis_connection = connect_redis("redis://127.0.0.1:6379");
    let mut matching_engine = MatchingEngine::init();
    matching_engine.recover_all_orderbooks(&mut redis_connection);

    let app_state = web::Data::new(AppState {
        matching_engine: Mutex::new(matching_engine),
        redis_connection: Mutex::new(redis_connection),
    });
    let worker_app_state = app_state.clone();
    let matching_engine = worker_app_state.matching_engine.lock().unwrap();
    let symbols = matching_engine.registered_exchanges();
    drop(matching_engine); // if not, then it will deadlock the matching_engine mutex
    symbols.par_iter().for_each(|symbol| {
        let sym4asks = symbol.clone();
        let asks_worker_app_state = worker_app_state.clone();
        let sym4bids = symbol.clone();
        let bids_worker_app_state = worker_app_state.clone();
        rayon::spawn(process_order(sym4bids, bids_worker_app_state, OrderSide::Bid));
        rayon::spawn(process_order(sym4asks, asks_worker_app_state, OrderSide::Ask));
    });
    let server = HttpServer::new(move || {
        App::new().service(
            scope("/api/v1")
                .app_data(app_state.clone())
                .service(health_check)
                .service(add_new_market)
                .service(get_asks)
                .service(get_bids)
                .service(get_quote)
        )
    })
        .listen(listener)?
        .run();

    Ok(server)
}

pub fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create client.");
    let mut connection = client.get_connection().expect("Could not connect to the client");
    return connection;
}

fn process_order(symbol: String, app_state: Data<AppState>, order_side: OrderSide) -> impl Fn() {
    move || {
        println!("{}s Worker Thread Created For {}", order_side.to_string(), symbol);
        loop {
            let con = &mut app_state.redis_connection.lock().unwrap();
            // right now two orders of different exchanges might be running sequentially instead of parallely-
            // since the entire matching engine gets locked for processing single order.
            // (maybe my guess), this can be made more performant by using mutex on each orderbook-
            // instead of locking complete matching engine,
            let mut matching_engine = app_state.matching_engine.lock().unwrap();
            let result = redis
                ::cmd("RPOP")
                .arg(format!("queues:{}:{}", order_side.to_string(), symbol))
                .query::<String>(con);
            match result {
                Ok(order_string) => {
                    println!(
                        "{} Order Recieved, poped it, symbol: {}",
                        order_side.to_string(),
                        symbol
                    );
                    matching_engine.process_order(&order_string, con);
                }
                Err(_) => {
                    println!("{}s Task queue empty, symbol: {}", order_side.to_string(), symbol);
                }
            }
        }
    }
}

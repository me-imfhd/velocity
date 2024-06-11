use std::{ collections::HashMap, net::TcpListener, sync::Mutex };

use actix_web::{ web::{ self, scope }, App, HttpServer };
use redis::{ Commands, Connection };

use crate::{
    config::GlobalConfig, db::schema::UserSchema, matching_engine::{ self, engine::MatchingEngine, users::Users, Id }, routes::{
        engine::{
            add_new_market,
            fill_limit_order,
            fill_market_order,
            get_asks,
            get_bids,
            get_quote,
            get_trades,
        },
        health::health_check,
        user::{ deposit, new_user, user_balance, withdraw },
    }
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
    pub users: Mutex<Users>,
    pub matching_engine: Mutex<MatchingEngine>,
    pub redis_connection: Mutex<Connection>,
}
async fn run(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Could not create client.");
    let mut connection = client.get_connection().expect("Could not connect to the client");
    let redis_connection = Mutex::new(connection);
    let app_state = web::Data::new(AppState {
        users: Mutex::new(Users::init()),
        matching_engine: Mutex::new(MatchingEngine::init()),
        redis_connection
    });
    let server = HttpServer::new(move || {
        App::new()
            .service(health_check)
            .service(
                scope("/api/v1")
                    .app_data(app_state.clone())
                    .service(
                        scope("/users")
                            .service(new_user)
                            .service(user_balance)
                            .service(deposit)
                            .service(withdraw)
                    )
                    .service(add_new_market)
                    .service(fill_limit_order)
                    .service(fill_market_order)
                    .service(get_trades)
                    .service(get_asks)
                    .service(get_bids)
                    .service(get_quote)
            )
    })
        .listen(listener)?
        .run();

    Ok(server)
}

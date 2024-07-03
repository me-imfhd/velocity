use std::{ net::TcpListener, sync::Mutex };

use actix_web::{ web::{ self, scope }, App, HttpServer };
use redis::Connection;

use crate::{
    config::GlobalConfig,
    db::ScyllaDb,
    routes::{ order::order, ping::ping, trades::trades, user::* },
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
    pub scylla_db: Mutex<ScyllaDb>,
    pub redis_connection: Mutex<Connection>,
    pub reqwest: Mutex<reqwest::Client>,
}
async fn run(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let uri = "127.0.0.1";
    let redis_uri = "redis://127.0.0.1:6379";
    let mut redis_connection = connect_redis(&redis_uri);
    let scylla_db = ScyllaDb::create_session(uri).await.unwrap();
    scylla_db.initialize().await.unwrap();
    let app_state = web::Data::new(AppState {
        scylla_db: Mutex::new(scylla_db),
        redis_connection: Mutex::new(redis_connection),
        reqwest: Mutex::new(reqwest::Client::new()),
    });
    let server = HttpServer::new(move || {
        App::new().service(
            scope("/api/v1")
                .app_data(app_state.clone())
                .service(ping)
                .service(order)
                .service(trades)
                .service(
                    scope("/user")
                        .service(new_user) // /new
                        .service(get_user) // ?id
                        .service(deposit) // /deposit
                        .service(withdraw) // /withdraw
                        .service(orders) // /orders
                )
        )
    })
        .listen(listener)?
        .run();

    Ok(server)
}

fn connect_redis(url: &str) -> Connection {
    let client = redis::Client::open(url).expect("Could not create client.");
    let mut connection = client.get_connection().expect("Could not connect to the client");
    return connection;
}

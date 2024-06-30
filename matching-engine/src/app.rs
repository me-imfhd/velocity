use std::{
    any,
    collections::HashMap,
    net::TcpListener,
    sync::{ mpsc::{ self, Sender }, Arc, Mutex, RwLock },
    thread,
};
use futures::executor::block_on;
use rayon::prelude::*;

use actix_web::{ web::{ self, scope, Data }, App, HttpServer };
use redis::{ Commands, Connection, Value };
use scylla::SessionBuilder;
use serde::{ Deserialize, Serialize };
use serde_json::from_str;

use crate::{
    config::GlobalConfig,
    routes::{ engine::{ get_asks, get_bids, get_quote }, health::ping },
    AppState,
};

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    pub async fn build(
        config: GlobalConfig,
        app_state: Data<AppState>
    ) -> Result<Self, std::io::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listner = TcpListener::bind(&address)?;
        let port = listner.local_addr().unwrap().port();
        let server = run(listner, app_state).await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
async fn run(
    listener: TcpListener,
    app_state: Data<AppState>
) -> Result<actix_web::dev::Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new().service(
            scope("/api/v1")
                .app_data(app_state.clone())
                .service(ping)
                .service(get_asks)
                .service(get_bids)
                .service(get_quote)
        )
    })
        .listen(listener)?
        .run();

    Ok(server)
}

use std::{ collections::HashMap, net::TcpListener, sync::Mutex };

use actix_web::{ web::{ self, scope }, App, HttpServer };
use serde::{ Deserialize, Serialize };

use crate::{
    config::GlobalConfig, routes::health::health_check,
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
}
async fn run(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {

    let app_state = web::Data::new(AppState {
    });
    let server = HttpServer::new(move || {
        App::new()
            .service(health_check)
            .service(
                scope("/api/v1") 
            )
    })
        .listen(listener)?
        .run();

    Ok(server)
}
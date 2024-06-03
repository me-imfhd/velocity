#![allow(unused)]
mod matching_engine;
use std::io::Result;

use matching_engine::orderbook::*;
use matching_engine::engine::*;
use matching_engine_rs::config;
use matching_engine_rs::app;
use matching_engine_rs::telemetry;
use rust_decimal_macros::dec;
#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config = config::get_config().expect("Failed to read config");
    let subscriber = telemetry::get_subscribers(config.clone().debug);
    telemetry::init_subscriber(subscriber);
    let application = app::Application::build(config).await?;

    tracing::event!(target: "matching_engine_rs", tracing::Level::INFO, "Listening on http://127.0.0.1:{}/", application.port());
    application.run_until_stopped().await?;
    Ok(())
}

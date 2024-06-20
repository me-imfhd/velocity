use std::{ borrow::BorrowMut, sync::Arc };

use actix_web::{ body, web::{ self, Data, Json, Query } };
use redis::Commands;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{ Deserialize, Serialize };

use crate::{
    app::{ self, AppState },
    matching_engine::{
        engine::Exchange,
        orderbook::{ Order, OrderSide, Price },
        Asset,
        Id,
        Quantity,
        Symbol,
    },
};
#[derive(Deserialize)]
struct SymbolStruct{
    symbol: Symbol
}
#[derive(Deserialize)]
struct NewMarket{
    base: Asset,
    quote: Asset
}
#[actix_web::post("/new_market")]
pub async fn add_new_market(
    body: Json<NewMarket>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let mut redis_connection = app_state.redis_connection.lock().unwrap();
    let exc = Exchange::new(body.base, body.quote);
    let exchange = matching_engine.add_new_market(exc.clone());
    if let Ok(matching_engine) = exchange {
        let symbol = exc.symbol;
        let bids_key = "orderbook:".to_string() + &symbol + ":bids";
        let asks_key = "orderbook:".to_string() + &symbol + ":asks";
        // initally bids and asks are empty vec
        redis
            ::cmd("MSET")
            .arg(bids_key)
            .arg("[]")
            .arg(asks_key)
            .arg("[]")
            .query::<String>(&mut redis_connection);
        return actix_web::HttpResponse::Ok().json("Created a new market successfully.");
    }
    actix_web::HttpResponse::Conflict().json(exchange.err())
}

#[derive(Deserialize)]
pub struct Quote {
    base: Asset,
    quote: Asset,
    order_side: OrderSide,
    quantity: Quantity,
}
#[actix_web::get("/quote")]
pub async fn get_quote(body: Query<Quote>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let quote = matching_engine.get_quote(
        &body.order_side,
        body.quantity,
        &Exchange::new(body.base, body.quote)
    );
    if let Ok(quote) = quote {
        return actix_web::HttpResponse::Ok().json(quote);
    }
    actix_web::HttpResponse::Conflict().json(quote.err())
}
#[actix_web::get("/asks")]
pub async fn get_asks(body: Query<SymbolStruct>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let exchange = Exchange::from_symbol(body.symbol.clone());
    let asks = matching_engine.get_asks(&exchange);
    if let Ok(asks) = asks {
        return actix_web::HttpResponse::Ok().json(asks);
    }
    actix_web::HttpResponse::Conflict().json(asks.err())
}
#[actix_web::get("/bids")]
pub async fn get_bids(body: Query<SymbolStruct>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let exchange = Exchange::from_symbol(body.symbol.clone());
    let bids = matching_engine.get_bids(&exchange);
    if let Ok(bids) = bids {
        return actix_web::HttpResponse::Ok().json(bids);
    }
    actix_web::HttpResponse::Conflict().json(bids.err())
}

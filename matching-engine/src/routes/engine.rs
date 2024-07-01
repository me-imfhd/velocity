use std::{ borrow::BorrowMut, sync::Arc };

use actix_web::{ body, web::{ self, Data, Json, Query }, HttpResponse };
use redis::Commands;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{ Deserialize, Serialize };

use crate::{
    matching_engine::{
        Asset, Exchange, Id, OrderSide, Quantity, Symbol
    }, AppState,
};
#[derive(Deserialize)]
struct SymbolStruct {
    symbol: Symbol,
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
pub async fn get_asks(
    body: Query<SymbolStruct>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let exchange_res = Exchange::from_symbol(body.symbol.clone());
    match exchange_res {
        Ok(exchange) => {
            let asks = matching_engine.get_asks(&exchange);
            if let Ok(asks) = asks {
                return actix_web::HttpResponse::Ok().json(asks);
            }
            actix_web::HttpResponse::Conflict().json(asks.err())
        }
        Err(_) => actix_web::HttpResponse::NotFound().json("Invalid Symbol"),
    }
}
#[actix_web::get("/bids")]
pub async fn get_bids(
    body: Query<SymbolStruct>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let exchange_res = Exchange::from_symbol(body.symbol.clone());
    match exchange_res {
        Ok(exchange) => {
            let bids = matching_engine.get_bids(&exchange);
            if let Ok(bids) = bids {
                return actix_web::HttpResponse::Ok().json(bids);
            }
            actix_web::HttpResponse::Conflict().json(bids.err())
        }
        Err(_) => actix_web::HttpResponse::NotFound().json("Invalid Symbol"),
    }
}

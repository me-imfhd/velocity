use std::{ borrow::BorrowMut, sync::Arc };

use actix_web::{ body, web::{ self, Data, Json, Query } };
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{ Deserialize, Serialize };

use crate::{
    app::{ self, AppState },
    matching_engine::{
        engine::Exchange,
        orderbook::{ Order, OrderSide, Price },
        users::Users,
        Asset,
        Id,
        Quantity,
    },
};

#[actix_web::post("/new_market")]
pub async fn add_new_market(
    body: Json<Exchange>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let exchange = matching_engine.add_new_market(body.0);
    if let Ok(matching_engine) = exchange {
        return actix_web::HttpResponse::Ok().json("Created a new market successfully.");
    }
    actix_web::HttpResponse::Conflict().json(exchange.err())
}
#[derive(Serialize)]
struct InsufficientBalanceResponse {
    message: String,
    asset: Asset,
    available_balance: Decimal,
    required_balance: Decimal,
}

#[derive(Deserialize)]
pub struct FillLimitOrder {
    price: Price,
    order_side: OrderSide,
    quantity: Quantity,
    user_id: Id,
    exchange: Exchange,
}
#[actix_web::post("/fill_limit_order")]
pub async fn fill_limit_order(
    body: Json<FillLimitOrder>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let mut users = app_state.users.lock().unwrap();
    let exists = users.does_exist(body.user_id);
    if exists == false {
        return actix_web::HttpResponse::NotFound().json("User Not Found");
    }
    match body.order_side {
        OrderSide::Ask => {
            let available_balance = users
                .open_balance(&body.exchange.base, body.user_id)
                .unwrap_or(dec!(0));
            if &body.quantity > &available_balance {
                return actix_web::HttpResponse::BadRequest().json(InsufficientBalanceResponse {
                    message: "Not enough available balance".to_string(),
                    asset: body.exchange.base,
                    required_balance: body.quantity,
                    available_balance,
                });
            } else {
                users.lock_amount(&body.exchange.base, body.user_id, body.quantity);
                println!(
                    "Locked Balance: {:?}",
                    users.locked_balance(&body.exchange.base, body.user_id)
                );
            }
        }
        OrderSide::Bid => {
            let available_balance = users
                .open_balance(&body.exchange.quote, body.user_id)
                .unwrap_or(dec!(0));
            if &body.price * body.quantity > available_balance {
                return actix_web::HttpResponse::BadRequest().json(InsufficientBalanceResponse {
                    message: "Not enough available balance".to_string(),
                    asset: body.exchange.quote,
                    required_balance: body.quantity * body.price,
                    available_balance,
                });
            } else {
                users.lock_amount(&body.exchange.quote, body.user_id, body.quantity * body.price);
                println!(
                    "Locked Balance: {:?}",
                    users.locked_balance(&body.exchange.quote, body.user_id)
                );
            }
        }
    }

    let price = body.price;
    let order_side = body.order_side.clone();
    let order = Order::new(order_side, body.quantity, true, body.user_id);
    // put this in the bidOrAsk queue for respective ticker e.g., BID:SOL, SELL:BTC
    let exchange = matching_engine.fill_limit_order(price, order, &mut users, &body.exchange);
    if let Ok(matching_engine) = exchange {
        return actix_web::HttpResponse::Ok().json("Ok");
    }
    actix_web::HttpResponse::Conflict().json(exchange.err())
}

#[derive(Deserialize)]
pub struct FillMarketOrder {
    order_side: OrderSide,
    quantity: Quantity,
    user_id: Id,
    exchange: Exchange,
}

#[actix_web::post("/fill_market_order")]
pub async fn fill_market_order(
    body: Json<FillMarketOrder>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let mut users = app_state.users.lock().unwrap();
    let exists = users.does_exist(body.user_id);
    let order_side = body.order_side.clone();
    let quote = matching_engine.get_quote(&order_side, body.quantity, &mut users, &body.exchange);
    if let Err(err) = quote {
        return actix_web::HttpResponse::BadRequest().json(err);
    }
    if exists == false {
        return actix_web::HttpResponse::NotFound().json("User Not Found");
    }
    match body.order_side {
        OrderSide::Ask => {
            let user_base_quantity = users
                .open_balance(&body.exchange.base, body.user_id)
                .ok()
                .unwrap();
            if &body.quantity > &user_base_quantity {
                return actix_web::HttpResponse::BadRequest().json(InsufficientBalanceResponse {
                    message: "Not enough available balance".to_string(),
                    asset: body.exchange.base,
                    required_balance: body.quantity,
                    available_balance: user_base_quantity,
                });
            }
        }
        OrderSide::Bid => {
            let user_quote_balance = users
                .open_balance(&body.exchange.quote, body.user_id)
                .ok()
                .unwrap();
            let quote_amount = quote.ok().unwrap();
            if quote_amount > user_quote_balance {
                return actix_web::HttpResponse::BadRequest().json(InsufficientBalanceResponse {
                    message: "Not enough available balance".to_string(),
                    asset: body.exchange.quote,
                    required_balance: quote_amount,
                    available_balance: user_quote_balance,
                });
            }
        }
    }
    let order = Order::new(order_side, body.quantity, false, body.user_id);
    // put this in the bidOrAsk queue for respective ticker e.g., BID:SOL, SELL:BTC
    let exchange = matching_engine.fill_market_order(order, &mut users, &body.exchange);
    if let Ok(matching_engine) = exchange {
        return actix_web::HttpResponse::Ok().json("Ok");
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
    let mut users = app_state.users.lock().unwrap();
    let quote = matching_engine.get_quote(
        &body.order_side,
        body.quantity,
        &mut users,
        &Exchange::new(body.base, body.quote)
    );
    if let Ok(quote) = quote {
        return actix_web::HttpResponse::Ok().json(quote);
    }
    actix_web::HttpResponse::Conflict().json(quote.err())
}
#[actix_web::get("/trades")]
pub async fn get_trades(
    body: Query<Exchange>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let trades = matching_engine.get_trades(&body.0);
    if let Ok(trades) = trades {
        return actix_web::HttpResponse::Ok().json(trades);
    }
    actix_web::HttpResponse::Conflict().json(trades.err())
}
#[actix_web::get("/asks")]
pub async fn get_asks(body: Query<Exchange>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let asks = matching_engine.get_asks(&body.0);
    if let Ok(asks) = asks {
        return actix_web::HttpResponse::Ok().json(asks);
    }
    actix_web::HttpResponse::Conflict().json(asks.err())
}
#[actix_web::get("/bids")]
pub async fn get_bids(body: Query<Exchange>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut matching_engine = app_state.matching_engine.lock().unwrap();
    let bids = matching_engine.get_bids(&body.0);
    if let Ok(bids) = bids {
        return actix_web::HttpResponse::Ok().json(bids);
    }
    actix_web::HttpResponse::Conflict().json(bids.err())
}

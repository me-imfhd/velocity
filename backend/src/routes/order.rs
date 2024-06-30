use std::time::Instant;

use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use futures::executor::block_on;
use redis::Value;
use serde::{ Deserialize, Serialize };
use serde_json::to_string;
use uuid::Uuid;

use crate::{
    app::AppState,
    db::schema::{ Asset, Exchange, Id, Order, OrderSide, OrderType, Price, Quantity, Symbol },
};

#[derive(Deserialize)]
pub struct OrderParams {
    price: Price,
    order_side: OrderSide,
    order_type: OrderType,
    quantity: Quantity,
    user_id: Id,
    base: Asset,
    quote: Asset,
}
// steps
// validate and lock balance {instant but varies}  ~3-7ms
// push the order to the queue for matching engine to process it {instant} less than ~0ms
// locally its takes ~5-7ms on avg
#[actix_web::post("/order")]
pub async fn order(body: Json<OrderParams>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let start = Instant::now();
    let exchange = Exchange::new(body.base, body.quote);
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
    // processing this by blocking the thread so that there is no race condition when locking balances
    let response = block_on(async move {
        let user = s_db.get_user(body.user_id).await;
        match user {
            Ok(mut user) => {
                match &body.order_side {
                    OrderSide::Bid => {
                        let ava_b = user.available_balance(&exchange.quote).unwrap();
                        if ava_b < body.price * body.quantity {
                            return HttpResponse::NotAcceptable().json("Insufficient balance.");
                        }
                        user.lock_amount(&body.quote, body.quantity * body.price);
                        s_db.update_user(&mut user).await;
                    }
                    OrderSide::Ask => {
                        let ava_b = user.available_balance(&exchange.base).unwrap();
                        if ava_b < body.quantity {
                            return HttpResponse::NotAcceptable().json("Insufficient balance.");
                        }
                        user.lock_amount(&body.base, body.quantity);
                        s_db.update_user(&mut user).await;
                    }
                }
                let order = Order::new(
                    0, // this field will be skipped
                    body.user_id,
                    body.quantity,
                    body.price,
                    body.order_side.clone(),
                    body.order_type.clone(),
                    exchange.symbol.clone()
                );
                let order_serialized = to_string(&order).unwrap();
                let res = redis
                    ::cmd("LPUSH") // this will place the order instantly for matching engine to process
                    .arg(format!("queues:{}", exchange.symbol))
                    .arg(order_serialized)
                    .query::<Value>(con);
                match res {
                    Ok(_) => HttpResponse::Ok().json("Order added."),
                    Err(err) =>
                        HttpResponse::InternalServerError().json(
                            err.to_string() + " Could not make order"
                        ),
                }
            }
            Err(err) => {
                return HttpResponse::NotFound().json(err.to_string());
            }
        }
    });
    let end = start.elapsed().as_millis();
    println!("Placed order in {} ms", end);
    response
}

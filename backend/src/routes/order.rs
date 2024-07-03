use std::time::Instant;

use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
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
// BENCHMARK: On an nice sunny day, its 1-3 ms
#[actix_web::post("/order")]
pub async fn order(body: Json<OrderParams>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let start = Instant::now();
    let exchange = Exchange::new(body.base, body.quote);
    let con = &mut app_state.redis_connection.lock().unwrap();
    let response = {
        let order = Order::new(
            0, // this field will be skipped when deserialzing
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
                HttpResponse::InternalServerError().json(err.to_string() + " Could not make order"),
        }
    };

    let end = start.elapsed().as_millis();
    println!("Time took: {} ms", end);
    response
}

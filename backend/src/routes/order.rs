use std::time::{ Duration, Instant };

use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use redis::Value;
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use serde_json::{ from_str, to_string };
use uuid::Uuid;

use crate::{
    app::AppState,
    db::schema::{
        Asset,
        Exchange,
        Id,
        Order,
        OrderId,
        OrderSide,
        OrderStatus,
        OrderType,
        Price,
        Quantity,
        Symbol,
    },
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
    let total_time = Instant::now();
    let exchange = Exchange::new(body.base, body.quote);
    let con = &mut app_state.redis_connection.lock().unwrap();
    let response = {
        let uuid = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
        let order = Order::new(
            uuid, // this field will be used to publish order response, and is not the actual order_id
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
            Ok(_) => {
                let end = start.elapsed().as_millis();
                println!("Placed order in {} ms", end);
                con.set_read_timeout(Some(Duration::from_millis(50)));
                let mut pubsub = con.as_pubsub();
                pubsub.subscribe(format!("{}", uuid)).unwrap();
                match pubsub.get_message() {
                    Ok(msg) => {
                        let response: String = msg.get_payload().unwrap();
                        match from_str::<OrderResponse>(&response) {
                            Ok(order_response) => { HttpResponse::Ok().json(order_response) }
                            Err(err) => HttpResponse::BadRequest().json(response),
                        }
                    }
                    Err(_) => HttpResponse::Accepted().json("Order is processing..."),
                }
            }
            Err(err) =>
                HttpResponse::InternalServerError().json(err.to_string() + " Could not make order"),
        }
    };

    let end = total_time.elapsed().as_millis();
    println!("Total time took: {} ms", end);
    response
}
#[derive(Serialize, Deserialize)]
pub struct OrderResponse {
    order_id: OrderId,
    quantity: Decimal,
    price: Decimal,
    executed_quantity: Decimal,
    executed_quote_quantity: Decimal,
    order_status: OrderStatus,
    order_type: OrderType,
    order_side: OrderSide,
    symbol: Symbol,
    timestamp: u64,
}

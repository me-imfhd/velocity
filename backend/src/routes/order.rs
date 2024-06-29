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
// steps
// ping/get last_user_id from matching engine {instant} ~5ms
// validate the order {instant but varies}  ~5ms-15ms
// push the order to the queue for matching engine to process it {instant} less than ~10ms
// if redis was up order is already placed succesfully, now save the order to the db {instant but varies} ~5-15ms
// if redis goes down, order does not get placed, if orderbook goes down use the db to replay the orders
// locally its takes ~20ms on avg
#[actix_web::post("/order")]
pub async fn order(body: Json<OrderParams>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let exchange = Exchange::new(body.base, body.quote);
    let reqwest = app_state.reqwest.lock().unwrap();
    let last_order_id = Uuid::new_v4();
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
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
                    println!("Locked Balance");
                }
                OrderSide::Ask => {
                    let ava_b = user.available_balance(&exchange.base).unwrap();
                    if ava_b < body.quantity {
                        return HttpResponse::NotAcceptable().json("Insufficient balance.");
                    }
                    user.lock_amount(&body.base, body.quantity);
                    s_db.update_user(&mut user).await;
                    println!("Locked Balance");
                }
            }
            let order = Order::new(
                last_order_id,
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
                    let result = s_db.new_order(order).await;
                    if result.is_err() {
                        println!("An Order was placed but was not able to save in database."); // analytics
                    }
                    match res {
                        Ok(_) => HttpResponse::Ok().json("Order added and saved."), // 99.9 percent
                        Err(err) => HttpResponse::Accepted().json("Order added."),
                    }
                }
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
}

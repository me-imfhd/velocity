use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use redis::Value;
use serde::{ Deserialize, Serialize };
use serde_json::to_string;

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

#[actix_web::post("/order")]
pub async fn order(body: Json<OrderParams>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
    let exchange = Exchange::new(body.base, body.quote);
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
        }
        Err(err) => {
            HttpResponse::NotFound().json(err.to_string());
        }
    }

    let result = s_db.new_order_id().await;
    match result {
        Ok(id) => {
            let order = Order::new(
                id,
                body.user_id,
                body.quantity,
                body.price,
                body.order_side.clone(),
                body.order_type.clone(),
                exchange.symbol.clone()
            );
            let order_serialized = to_string(&order).unwrap();
            let result = s_db.new_order(order).await;
            match result {
                Ok(_) => {
                    let res = redis
                        ::cmd("LPUSH")
                        .arg(format!("queues:{}", exchange.symbol))
                        .arg(order_serialized)
                        .query::<Value>(con);
                    match res {
                        Ok(_) => HttpResponse::Accepted().json("Order added."),
                        Err(err) =>
                            HttpResponse::InternalServerError().json(
                                err.to_string() + " Could not make order"
                            ),
                    }
                }
                Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
            }
        }
        Err(err) =>
            HttpResponse::InternalServerError().json(err.to_string() + " Could not make order"),
    }
}

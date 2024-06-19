use actix_web::{ web::{ Data, Json }, HttpResponse };
use redis::Value;
use serde::Deserialize;
use serde_json::to_string;

use crate::{
    app::AppState,
    db::schema::{ Id, Order, OrderSide, OrderType, Price, Quantity, Symbol },
};

#[derive(Deserialize)]
pub struct OrderParams {
    price: Price,
    order_side: OrderSide,
    order_type: OrderType,
    quantity: Quantity,
    user_id: Id,
    symbol: Symbol,
}
#[actix_web::post("/order")]
pub async fn order(
    body: Json<OrderParams>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
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
                body.symbol.to_string()
            );
            let order_serialized = to_string(&order).unwrap();
            let result = s_db.new_order(order).await;
            match result {
                Ok(_) => {
                    let res = redis
                        ::cmd("LPUSH")
                        .arg(format!("queues:{}:{}", body.order_side.to_string(), body.symbol))
                        .arg(order_serialized)
                        .query::<Value>(con);
                    match res {
                        Ok(_) => HttpResponse::Accepted().json("Order added."),
                        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
                    }
                }
                Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

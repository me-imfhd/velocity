use std::time::{ Duration, Instant };

use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use redis::{ Commands, PubSub, Value };
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use serde_json::{ from_str, to_string };
use uuid::Uuid;
use super::*;

use crate::{
    app::AppState,
    db::{get_epoch_micros, schema::{
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
    }},
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
pub async fn execute_order(
    body: Json<OrderParams>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let placed_order_time = Instant::now();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    // Todo: Validate the symbol
    let symbol = body.symbol.clone();
    let response = {
        let order = Order::new(
            sub_id, // this field will be used to publish order response, and is not the actual order_id
            body.user_id,
            body.quantity,
            body.price,
            body.order_side.clone(),
            body.order_type.clone(),
            symbol.clone()
        );
        let req = to_string(&EngineRequests::ExecuteOrder(order)).unwrap();
        let res = redis
            ::cmd("LPUSH")
            .arg(format!("queues:{}", symbol))
            .arg(req)
            .query::<Value>(con);
        println!("Placed order in {}ms", placed_order_time.elapsed().as_millis());
        let recieved_time = Instant::now();
        match res {
            Ok(_) => {
                let mut response_result: Option<String> = None;
                loop {
                    let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                    if let Ok(response) = result {
                        response_result = Some(response);
                        break;
                    }
                }
                let response: String = response_result.unwrap();
                println!("Recieved order response in: {} ms", recieved_time.elapsed().as_millis());
                match from_str::<Order>(&response) {
                    Ok(response) => { HttpResponse::Ok().json(response) }
                    Err(err) => HttpResponse::BadRequest().json(response),
                }
            }
            Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        }
    };

    response
}

#[actix_web::delete("/orders")]
pub async fn order_cancel_all(
    mut body: Json<CancelAll>,
    app_state: Data<AppState>
) -> HttpResponse {
    let total_time = Instant::now();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let symbol = body.symbol.clone();
    let response = {
        body.sub_id = sub_id;
        body.timestamp = get_epoch_micros() as i64;
        let req = to_string(&EngineRequests::CancelAll(body.0)).unwrap();
        let res = redis
            ::cmd("LPUSH")
            .arg(format!("queues:{}", symbol))
            .arg(req)
            .query::<Value>(con);
        match res {
            Ok(_) => {
                let mut response_result: Option<String> = None;
                loop {
                    let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                    if let Ok(response) = result {
                        response_result = Some(response);
                        break;
                    }
                }
                let response: String = response_result.unwrap();
                match from_str::<Vec<Order>>(&response) {
                    Ok(response) => { HttpResponse::Ok().json(response) }
                    Err(err) => HttpResponse::BadRequest().json(response),
                }
            }
            Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        }
    };

    let end = total_time.elapsed().as_millis();
    println!("Total time took: {} ms", end);
    response
}
#[actix_web::delete("/order")]
pub async fn order_cancel(mut body: Json<CancelOrder>, app_state: Data<AppState>) -> HttpResponse {
    let total_time = Instant::now();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let symbol = body.symbol.clone();
    let response = {
        body.sub_id = sub_id;
        body.timestamp = get_epoch_micros() as i64;
        let req = to_string(&EngineRequests::CancelOrder(body.0)).unwrap();
        let res = redis
            ::cmd("LPUSH")
            .arg(format!("queues:{}", symbol))
            .arg(req)
            .query::<Value>(con);
        match res {
            Ok(_) => {
                let mut response_result: Option<String> = None;
                loop {
                    let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                    if let Ok(response) = result {
                        response_result = Some(response);
                        break;
                    }
                }
                let response: String = response_result.unwrap();
                match from_str::<Order>(&response) {
                    Ok(response) => { HttpResponse::Ok().json(response) }
                    Err(err) => HttpResponse::BadRequest().json(response),
                }
            }
            Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        }
    };

    let end = total_time.elapsed().as_millis();
    println!("Total time took: {} ms", end);
    response
}

#[actix_web::get("/order")]
pub async fn get_open_order(mut body: Json<OpenOrder>, app_state: Data<AppState>) -> HttpResponse {
    let total_time = Instant::now();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let symbol = body.symbol.clone();
    let response = {
        body.sub_id = sub_id;
        let req = to_string(&EngineRequests::OpenOrder(body.0)).unwrap();
        let res = redis
            ::cmd("LPUSH")
            .arg(format!("queues:{}", symbol))
            .arg(req)
            .query::<Value>(con);
        match res {
            Ok(_) => {
                let mut response_result: Option<String> = None;
                loop {
                    let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                    if let Ok(response) = result {
                        response_result = Some(response);
                        break;
                    }
                }
                let response: String = response_result.unwrap();
                match from_str::<Order>(&response) {
                    Ok(response) => { HttpResponse::Ok().json(response) }
                    Err(err) => HttpResponse::BadRequest().json(response),
                }
            }
            Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        }
    };

    let end = total_time.elapsed().as_millis();
    println!("Total time took: {} ms", end);
    response
}

#[actix_web::get("/orders")]
pub async fn get_open_orders(
    mut body: Json<OpenOrders>,
    app_state: Data<AppState>
) -> HttpResponse {
    let total_time = Instant::now();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let symbol = body.symbol.clone();
    let response = {
        body.sub_id = sub_id;
        let req = to_string(&EngineRequests::OpenOrders(body.0)).unwrap();
        let res = redis
            ::cmd("LPUSH")
            .arg(format!("queues:{}", symbol))
            .arg(req)
            .query::<Value>(con);
        match res {
            Ok(_) => {
                let mut response_result: Option<String> = None;
                loop {
                    let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                    if let Ok(response) = result {
                        response_result = Some(response);
                        break;
                    }
                }
                let response: String = response_result.unwrap();
                match from_str::<Vec<Order>>(&response) {
                    Ok(response) => { HttpResponse::Ok().json(response) }
                    Err(err) => HttpResponse::BadRequest().json(response),
                }
            }
            Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        }
    };

    let end = total_time.elapsed().as_millis();
    println!("Total time took: {} ms", end);
    response
}

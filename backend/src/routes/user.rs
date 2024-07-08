use std::time::Duration;

use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use redis::{ Commands, Value };
use serde::{ Deserialize, Serialize };
use serde_json::{ from_str, to_string };
use super::*;
use crate::{ api::user, app::AppState, db::schema::{ Asset, Id, Quantity, User } };

#[actix_web::post("/new")]
pub async fn new_user(app_state: Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let req = to_string(
        &UserRequests::NewUser(NewUser {
            sub_id,
        })
    ).unwrap();
    let response = redis::cmd("LPUSH").arg("queues:user").arg(req).query::<Value>(con);
    match response {
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
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.new_user(user.clone()).await;
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(response),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::get("")]
pub async fn get_user(
    mut query: Query<GetUserBalances>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    query.sub_id = sub_id;
    let req = to_string(&UserRequests::GetUserBalances(query.0)).unwrap();
    let response = redis::cmd("LPUSH").arg("queues:user").arg(req).query::<Value>(&mut con);
    match response {
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
            match from_str::<User>(&response) {
                Ok(user) => {
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(response),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::post("/deposit")]
pub async fn deposit(
    mut body: Json<Deposit>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    body.sub_id = sub_id;
    let req = to_string(&UserRequests::Deposit(body.0)).unwrap();
    let response = redis::cmd("LPUSH").arg("queues:user").arg(req).query::<Value>(con);
    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg(sub_id).query::<String>(con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.update_user(&mut user.clone()).await;
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(response),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::post("/withdraw")]
pub async fn withdraw(
    mut body: Json<Withdraw>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    body.sub_id = sub_id;
    let req = to_string(&UserRequests::Withdraw(body.0)).unwrap();
    let response = redis::cmd("LPUSH").arg("queues:user").arg(req).query::<Value>(con);
    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg(sub_id).query::<String>(con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.update_user(&mut user.clone()).await;
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(response),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[derive(Serialize, Deserialize)]
struct UserOrders {
    user_id: Id,
}
#[actix_web::get("/history/orders")]
pub async fn orders_history(
    query: Query<UserOrders>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let result = s_db.get_user(query.user_id).await;
    match result {
        Ok(user) => {
            let user_orders = s_db.get_users_orders(query.user_id).await;
            match user_orders {
                Ok(user_orders) => HttpResponse::Ok().json(user_orders),
                Err(err) =>
                    HttpResponse::NotFound().json(
                        format!("Orders Not Found\n {}", err.to_string())
                    ),
            }
        }
        Err(err) => HttpResponse::NotFound().json(format!("User Not Found\n {}", err.to_string())),
    }
}

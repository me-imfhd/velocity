use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use serde::{ Deserialize, Serialize };

use crate::{ api::user, app::AppState, db::schema::{ Asset, Id, Quantity, User } };

#[actix_web::post("/new")]
pub async fn new_user(app_state: Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let result = s_db.new_user_id().await;
    match result {
        Ok(id) => {
            let u = User::new(id);
            let user_created = u.clone();
            let user = s_db.new_user(u).await;
            match user {
                Ok(_) => HttpResponse::Created().json(user_created),
                Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[derive(Serialize, Deserialize)]
struct QueryId {
    id: Id,
}
#[actix_web::get("")]
pub async fn get_user(query: Query<QueryId>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let result = s_db.get_user(query.id).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::NotFound().json(format!("User Not Found\n {}", err.to_string())),
    }
}

#[derive(Serialize, Deserialize)]
struct Deposit {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
}
#[actix_web::post("/deposit")]
pub async fn deposit(body: Json<Deposit>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let mut result = s_db.get_user(body.user_id).await;
    match result {
        Ok(mut user) => {
            user.deposit(&body.asset, body.quantity);
            s_db.update_user(&mut user).await.unwrap();
            HttpResponse::Ok().json(user)
        }
        Err(err) => HttpResponse::NotFound().json(format!("User Not Found\n {}", err.to_string())),
    }
}

#[derive(Serialize, Deserialize)]
struct UserOrders {
    user_id: Id,
}
#[actix_web::get("/orders")]
pub async fn orders(query: Query<UserOrders>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let result = s_db.get_user(query.user_id).await;
    match result {
        Ok(user) => {
           let user_orders = s_db.get_users_orders(query.user_id).await;
           match user_orders {
            Ok(user_orders) => HttpResponse::Ok().json(user_orders),
            Err(err) =>  HttpResponse::NotFound().json(format!("Orders Not Found\n {}", err.to_string())),
           } 
        }
        Err(err) => HttpResponse::NotFound().json(format!("User Not Found\n {}", err.to_string())),
    }
}

use actix_web::{ web::{ Data, Json, Query }, HttpResponse };
use serde::{ Deserialize, Serialize };

use crate::{ api::user, app::AppState, db::schema::{ Asset, Id, Quantity, User } };

#[actix_web::post("/new")]
pub async fn new_user(app_state: Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let reqwest = app_state.reqwest.lock().unwrap();
    let response = reqwest.post("http://127.0.0.1:5000/api/v1/user/new").send().await;
    match response {
        Ok(response) => {
            let id: i64 = response.json().await.unwrap();
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
    let reqwest = app_state.reqwest.lock().unwrap();
    if let Err(err) = s_db.get_user(body.user_id).await {
        return HttpResponse::NotFound().json(format!("User Not Found\n {}", err));
    }

    if
        let Ok(response) = reqwest
            .post("http://127.0.0.1:5000/api/v1/user/deposit")
            .json(&body.0)
            .send().await
    {
        let mut user: User = response.json().await.unwrap();
        s_db.update_user(&mut user).await.unwrap();
        return HttpResponse::Ok().json(user);
    }

    HttpResponse::InternalServerError().json("Try again later, matching engine is down")
}
#[derive(Serialize, Deserialize)]
struct Withdraw {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
}
#[actix_web::post("/withdraw")]
pub async fn withdraw(body: Json<Withdraw>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let reqwest = app_state.reqwest.lock().unwrap();
    let mut result = s_db.get_user(body.user_id).await;
    match result {
        Ok(mut user) => {
            if body.quantity > user.available_balance(&body.asset).unwrap() {
                return HttpResponse::NotAcceptable().json("Over Withdrawing");
            }
            if
                let Ok(response) = reqwest
                    .post("http://127.0.0.1:5000/api/v1/user/withdraw")
                    .json(&body.0)
                    .send().await
            {
                let mut user: User = response.json().await.unwrap();
                s_db.update_user(&mut user).await.unwrap();
                return HttpResponse::Ok().json(user);
            } else {
                return HttpResponse::InternalServerError().json(
                    "Try again later, matching engine is down"
                );
            }
        }
        Err(err) => HttpResponse::NotFound().json(format!("User Not Found\n {}", err.to_string())),
    }
}

#[derive(Serialize, Deserialize)]
struct UserOrders {
    user_id: Id,
}
#[actix_web::get("/orders")]
pub async fn orders(
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

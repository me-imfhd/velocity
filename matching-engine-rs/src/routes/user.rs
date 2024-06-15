use actix_web::web::{ self, Data };
use config::Value;
use serde::{ Deserialize, Serialize };
use serde_json::to_string;
use crate::{
    app::AppState,
    matching_engine::{ error::MatchingEngineErrors, Asset, AssetIter, Id, Quantity },
};

#[actix_web::post("/new")]
pub async fn new_user(app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut users = app_state.users.lock().unwrap();
    let mut redis_connection = app_state.redis_connection.lock().unwrap();
    let user_id = users.new_user();
    let user = users.users.get(&user_id).unwrap();
    let user_str = to_string(user).unwrap();
    let res = redis
        ::cmd("SET")
        .arg(format!("users:{}", user_id))
        .arg(user_str)
        .query::<String>(&mut redis_connection);
    if res.is_err() {
        users.users.remove(&user_id);
        return actix_web::HttpResponse
            ::InternalServerError()
            .json("Could not set user key value pair");
    }
    actix_web::HttpResponse::Ok().json(user_id)
}

#[derive(Deserialize)]
struct UserBalanceQuery {
    id: Id,
    asset: Asset,
}

#[actix_web::get("/balance")]
pub async fn user_balance(
    queries: web::Query<UserBalanceQuery>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut users = app_state.users.lock().unwrap();
    let balance = users.balance(&queries.asset, queries.id);
    match balance {
        Ok(balance) => actix_web::HttpResponse::Ok().json(balance),
        Err(e) => actix_web::HttpResponse::NotFound().json(e),
    }
}
#[derive(Deserialize)]
struct Deposit {
    id: Id,
    asset: Asset,
    quantity: Quantity,
}
#[actix_web::post("/deposit")]
pub async fn deposit(
    body: web::Json<Deposit>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut users = app_state.users.lock().unwrap();
    let mut redis_connection = app_state.redis_connection.lock().unwrap();
    let res = users.deposit(&body.asset, body.quantity, body.id);
    match res {
        Ok(user) => {
            let res = redis
                ::cmd("SET")
                .arg(format!("users:{}", user.id))
                .arg(to_string(user).unwrap())
                .query::<String>(&mut redis_connection);
            if res.is_err() {
                users.withdraw(&body.asset, body.quantity, body.id);
                return actix_web::HttpResponse
                    ::InternalServerError()
                    .json("Could not set user key value pair");
            }
            return actix_web::HttpResponse::Ok().json("Deposited");
        }
        Err(e) => actix_web::HttpResponse::NotFound().json(e),
    }
}

#[derive(Deserialize)]
struct Withdraw {
    id: Id,
    asset: Asset,
    quantity: Quantity,
}

#[derive(Serialize)]
struct OverWithdrawing {
    message: String,
    available_balance: Quantity,
    locked_balance: Quantity,
    total_balance: Quantity,
}
#[actix_web::post("/withdraw")]
pub async fn withdraw(
    body: web::Json<Withdraw>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut users = app_state.users.lock().unwrap();
    let mut redis_connection = app_state.redis_connection.lock().unwrap();
    let available_balance = users.open_balance(&body.asset, body.id);
    match available_balance {
        Ok(available_balance) => {
            if body.quantity > available_balance {
                return actix_web::HttpResponse::BadRequest().json(OverWithdrawing {
                    message: "OverWithdrawing".to_string(),
                    available_balance,
                    locked_balance: *users.locked_balance(&body.asset, body.id).unwrap(),
                    total_balance: *users.balance(&body.asset, body.id).unwrap(),
                });
            }
            let res = users.withdraw(&body.asset, body.quantity, body.id);
            match res {
                Ok(user) => {
                    let res = redis
                        ::cmd("SET")
                        .arg(format!("users:{}", user.id))
                        .arg(to_string(user).unwrap())
                        .query::<String>(&mut redis_connection);
                    if res.is_err() {
                        users.deposit(&body.asset, body.quantity, body.id);
                        return actix_web::HttpResponse
                            ::InternalServerError()
                            .json("Could not set user key value pair");
                    }
                    return actix_web::HttpResponse::Ok().json("Withdrawn");
                }
                Err(e) => actix_web::HttpResponse::NotFound().json(e),
            }
        }
        Err(err) => actix_web::HttpResponse::NotFound().json(err),
    }
}

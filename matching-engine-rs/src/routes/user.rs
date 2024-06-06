use actix_web::web::{ self, Data };
use serde::{Deserialize, Serialize};
use crate::{
    app::AppState,
    matching_engine::{ error::MatchingEngineErrors, Asset, AssetIter, Id, Quantity },
};

#[actix_web::post("/new")]
pub async fn new_user(app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut users = app_state.users.lock().unwrap();
    let user_id = users.new_user();
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
    let res = users.deposit(&body.asset, body.quantity, body.id);
    match res {
        Ok(res) => actix_web::HttpResponse::Ok().json("Deposited"),
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
    let balance = users.balance(&body.asset, body.id);
    if let Err(err) = balance {
        return actix_web::HttpResponse::BadRequest().json(err);
    }
    let total_balance = balance.unwrap();
    let locked_balance = users.locked_balance(&body.asset, body.id).unwrap();
    let available_balance = total_balance - locked_balance;
    if body.quantity > available_balance {
        return actix_web::HttpResponse::BadRequest().json(OverWithdrawing{
            message: "OverWithdrawing".to_string(),
            available_balance,
            locked_balance: *locked_balance,
            total_balance: *total_balance,
        });
    }
    let res = users.withdraw(&body.asset, body.quantity,
     body.id);
    match res {
        Ok(res) => actix_web::HttpResponse::Ok().json("Withdrawn"),
        Err(e) => actix_web::HttpResponse::NotFound().json(e),
    }
}

use actix_web::web::{ self, Data };
use config::Value;
use serde::{ Deserialize, Serialize };
use serde_json::to_string;
use crate::{
    matching_engine::{ error::MatchingEngineErrors, Asset, AssetIter, Id, Quantity },
    AppState,
};

#[actix_web::post("/new")]
pub async fn new_user(app_state: Data<AppState>) -> actix_web::HttpResponse {
    let mut me = app_state.matching_engine.lock().unwrap();
    let total_users = me.users.users.len() as u64;
    let user_id = total_users + 1;
    me.users.new_user(user_id);
    println!("New user created id: {}", user_id);
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
    let mut users = &mut app_state.matching_engine.lock().unwrap().users;
    let balance = users.balance(&queries.asset, queries.id);
    match balance {
        Ok(balance) => actix_web::HttpResponse::Ok().json(balance),
        Err(e) => actix_web::HttpResponse::NotFound().json(e),
    }
}
#[derive(Deserialize)]
struct Deposit {
    user_id: Id,
    asset: Asset,
    quantity: Quantity,
}
#[actix_web::post("/deposit")]
pub async fn deposit(
    body: web::Json<Deposit>,
    app_state: Data<AppState>
) -> actix_web::HttpResponse {
    let mut users = &mut app_state.matching_engine.lock().unwrap().users;
    users.deposit(&body.asset, body.quantity, body.user_id).unwrap();
    let user = users.users.get(&body.user_id).unwrap();
    println!("Deposited balance");
    actix_web::HttpResponse::Ok().json(user.clone())
}

#[derive(Deserialize)]
struct Withdraw {
    user_id: Id,
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
    let mut users = &mut app_state.matching_engine.lock().unwrap().users;
    users.withdraw(&body.asset, body.quantity, body.user_id);
    let user = users.users.get(&body.user_id).unwrap();
    println!("Withdrawn balance");
    actix_web::HttpResponse::Ok().json(user.clone())
}

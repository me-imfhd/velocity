use actix_web::{ web::{ Data, Query }, HttpResponse };
use serde::{ Deserialize, Serialize };

use crate::{ app::AppState, db::schema::{ Exchange, Symbol } };

#[derive(Serialize, Deserialize)]
struct Trades {
    symbol: Symbol,
}
#[actix_web::get("/trades")]
pub async fn trades(query: Query<Trades>, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let exchange = Exchange::from_symbol(query.symbol.clone());
    match exchange {
        Ok(exchange) => {
            let result = s_db.get_trades(exchange.symbol).await;
            match result {
                Ok(trades) => { HttpResponse::Ok().json(trades) }
                Err(err) =>
                    HttpResponse::NotFound().json(format!("Trades Not Found\n {}", err.to_string())),
            }
        }
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

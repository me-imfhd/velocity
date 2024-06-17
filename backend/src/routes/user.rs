use std::borrow::Borrow;

use actix_web::{ web::Data, HttpResponse };

use crate::{ app::AppState, db::schema::UserSchema };

#[actix_web::post("/new_user")]
pub async fn new_user(app_state: Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let u = UserSchema::new();
    let user_created = u.clone();
    let user = s_db.new_user(u).await;
    match user {
        Ok(user) => HttpResponse::Created().json(user_created),
        Err(_) => HttpResponse::InternalServerError().json("Database is down"),
    }
}

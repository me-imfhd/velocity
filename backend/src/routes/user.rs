use actix_web::{ web::Data, HttpResponse };

use crate::{ app::AppState, db::schema::User };

#[actix_web::post("/new_user")]
pub async fn new_user(app_state: Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let user_id = s_db.new_user_id().await.unwrap();
    let u = User::new(user_id);
    let user_created = u.clone();
    let user = s_db.new_user(u).await;
    match user {
        Ok(_) => HttpResponse::Created().json(user_created),
        Err(_) => HttpResponse::InternalServerError().json("Database is down"),
    }
}

use std::{ borrow::BorrowMut, sync::Arc };

#[actix_web::get("/ping")]
pub async fn ping() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json("Pong.")
}

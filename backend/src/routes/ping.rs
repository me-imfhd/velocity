use std::{ borrow::BorrowMut, sync::Arc };

use actix_web::web;

use crate::app::{ self, AppState };

#[actix_web::get("/ping")]
pub async fn ping() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json("pong")
}

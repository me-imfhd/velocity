use std::{borrow::BorrowMut, sync::Arc};

use actix_web::web;

use crate::app::{self, AppState};

#[actix_web::get("/health-check")]
pub async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json("App is healthy.")
}
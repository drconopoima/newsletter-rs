use actix_web::{HttpResponse, Responder};

pub async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

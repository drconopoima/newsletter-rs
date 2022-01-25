use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

#[derive(serde::Deserialize)]
struct SubscriptionFormData {
    email: String,
    name: String,
}

async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

async fn subscription(_form: web::Form<SubscriptionFormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

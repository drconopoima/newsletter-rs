use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

async fn subscription() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/healthcheck", web::get().to(healthcheck))
            .route("/subscription", web::post().to(subscription))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

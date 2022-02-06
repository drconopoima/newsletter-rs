use crate::{
    postgres::NoTlsPostgresConnection,
    routes::{healthcheck, subscription},
};
use actix_web::{dev::Server, web, App, HttpServer};
use std::net::TcpListener;
use std::sync::Arc;

pub fn run(
    listener: TcpListener,
    postgres_connection: NoTlsPostgresConnection,
) -> Result<Server, std::io::Error> {
    let postgres_connection = Arc::new(postgres_connection);
    let server = HttpServer::new(move || {
        App::new()
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(postgres_connection.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

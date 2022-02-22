use crate::routes::{healthcheck, subscription};
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpServer};
use deadpool_postgres::Pool;
use std::net::TcpListener;
use std::sync::Arc;

pub fn run(listener: TcpListener, postgres_pool: Pool) -> Result<Server, std::io::Error> {
    let postgres_pool = Arc::new(postgres_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Logging middleware
            .wrap(Logger::default())
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(postgres_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

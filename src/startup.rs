use crate::routes::{healthcheck, subscription};
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpServer};
use deadpool_postgres::Pool;
use std::net::TcpListener;
use std::sync::Arc;

pub fn run(
    listener: TcpListener,
    postgres_pool: Pool,
    admin_bind_address: Option<(String, u16)>,
) -> Result<(Server, Option<Server>), std::io::Error> {
    let postgres_pool = Arc::new(postgres_pool);
    if admin_bind_address.is_none() {
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
        return Ok((server, None));
    }
    let admin_bind_address = admin_bind_address.unwrap();
    let admin_listener = TcpListener::bind(admin_bind_address)?;
    let postgres_pool1 = postgres_pool.clone();
    let server1 = HttpServer::new(move || {
        App::new()
            // Logging middleware
            .wrap(Logger::default())
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(postgres_pool1.clone())
    })
    .listen(listener)?
    .run();
    let server2 = HttpServer::new(move || {
        App::new()
            // Logging middleware
            .wrap(Logger::default())
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Register the Postgres connection as part of application state
            .app_data(postgres_pool.clone())
    })
    .listen(admin_listener)?
    .run();
    Ok((server1, Some(server2)))
}

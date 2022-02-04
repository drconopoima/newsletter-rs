use crate::routes::{healthcheck, subscription};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tokio_postgres::{Client, Error};

async fn establish_pg_connection(pg_connection_string: String) -> Result<Client, Error> {
    let (client, connection) =
        tokio_postgres::connect(&pg_connection_string, tokio_postgres::NoTls)
            .await
            .unwrap_or_else(|_| {
                panic!("ERROR: Failed to connect to Postgres at URL: {}",
                &pg_connection_string
            )});
    // Spawn connection
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            panic!(
                "Connection error with postgres at '{}', {}",
                &pg_connection_string, error
            );
        }
    });
    Ok(client)
}

pub fn run(listener: TcpListener, pg_connection_string: String) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(establish_pg_connection(pg_connection_string.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

use crate::routes::{healthcheck, subscription};
use actix_web::{App, dev::Server, HttpServer, web};
use std::net::TcpListener;
use tokio_postgres::{Client,NoTls};

async fn connect(pg_connection_string: String) -> Client {
    let (client, connection) =
        tokio_postgres::connect(&pg_connection_string, NoTls)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "ERROR: Failed to connect to Postgres at URL: {}",
                    &pg_connection_string
                )
            });
    // Spawn connection
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            panic!(
                "Connection error with postgres at '{}', {}",
                &pg_connection_string, error
            );
        }
    });
    client
}

pub fn run(listener: TcpListener, pg_connection_string: String) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            // Register the Postgres connection as part of application state
            .app_data(web::Data::new(connect(pg_connection_string.clone())))
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

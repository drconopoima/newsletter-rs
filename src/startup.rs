use crate::routes::{healthcheck,subscription};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tokio_postgres::{connect,Client,Connection,Error,Socket};
use tokio_postgres::tls::NoTlsStream;

type DBCon = Connection<Socket,NoTlsStream>;

pub async fn establish_pg_connection(connection_string:String) -> Result<(Client, DBCon),Error> {
    let (client, connection) =
        connect(&connection_string, tokio_postgres::NoTls)
            .await
            .expect(&format!(
                "ERROR: Failed to connect to Postgres at URL: {}",
                &connection_string
            ));
    Ok((client,connection))
}


pub fn run(listener: TcpListener, connection_string: String) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            // Ensure App to be running correctly
            .route("/healthcheck", web::get().to(healthcheck))
            // Handle newsletter subscription requests
            .route("/subscription", web::post().to(subscription))
            // Register the Postgres connection as part of application state
            .app_data(establish_pg_connection(connection_string.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

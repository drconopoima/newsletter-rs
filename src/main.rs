use deadpool_postgres::Pool;
use newsletter_rs::{
    configuration::{get_configuration, Settings},
    postgres::generate_connection_pool,
    startup::run,
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_file: &str = "configuration";
    let configuration: Settings = get_configuration(config_file).unwrap_or_else(|error| {
        panic!(
            "ERROR: Failed to read configuration file \"{}\", {}.",
            config_file, error
        )
    });
    let bind_address: (&str, u16) = ("127.0.0.1", configuration.application_port);
    // Raises if failed to bind address
    let listener = TcpListener::bind(bind_address)?;
    let postgres_connection: Pool =
        generate_connection_pool(configuration.database.connection_string());
    // Run server on TcpListener
    run(listener, postgres_connection)?.await
}

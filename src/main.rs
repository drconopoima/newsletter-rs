use newsletter_rs::configuration::{get_configuration, Settings};
use newsletter_rs::startup::run;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //
    let config_file: &str = "configuration";
    let configuration: Settings = get_configuration(config_file).unwrap_or_else(
        |error| {
            panic!("ERROR: Failed to read configuration file \"{}\", {}.",
                config_file, error)
        });
    let bind_address: (&str, u16) = ("127.0.0.1", configuration.application_port);
    // Raises if failed to bind address
    let listener = TcpListener::bind(bind_address)?;
    // Run server on TcpListener
    run(listener)?.await
}

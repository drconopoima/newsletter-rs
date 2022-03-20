use actix_web::dev::Server;
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use futures::future;
use newsletter_rs::{
    configuration::{get_configuration, ApplicationSettings, DatabaseSettings},
    postgres::{check_database_exists, generate_connection_pool, migrate_database},
    startup::run,
    telemetry,
};
use std::net::TcpListener;
use std::time::Duration;

#[actix_web::main]
async fn main() -> Result<()> {
    let subscriber_name = env!("CARGO_PKG_NAME");
    let env_filter = "info";
    let subscriber = telemetry::get_subscriber(
        subscriber_name.to_owned(),
        env_filter.to_owned(),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber).with_context(|| format!("{}::main: Failed to initialize tracing subscriber with name '{}' and filter level '{}'", env!("CARGO_PKG_NAME"), subscriber_name, env_filter))?;
    let config_file: &str = "configuration.yaml";
    let configuration: ApplicationSettings =
        get_configuration(config_file).unwrap_or_else(|error| {
            panic!(
                "ERROR: Failed to read configuration file \"{}\", {}.",
                config_file, error
            )
        });
    let connection_string = &configuration.database.connection_string().to_owned();
    let database_name = match configuration.database.database.as_ref() {
        Some(database_name) => database_name.to_owned(),
        _ => {
            let database_name = "newsletter".to_owned();
            tracing::warn!(
                "Failed to retrieve a database name from settings, using default value '{}'",
                database_name
            );
            database_name.to_owned()
        }
    };
    let database_settings = DatabaseSettings {
        port: configuration.database.port,
        host: configuration.database.host.to_owned(),
        username: configuration.database.username.to_owned(),
        password: configuration.database.password.to_owned(),
        database: Some(database_name.to_owned()),
    };
    let postgres_connection: Pool = match configuration.database_migration {
        Some(ref migration_settings) => {
            if migration_settings.migrate {
                let mut folder = "./migrations".to_owned();
                if let Some(ref migration_folder) = migration_settings.folder {
                    folder = migration_folder.to_owned();
                }
                migrate_database(database_settings, folder).await
            } else {
                generate_connection_pool(connection_string.to_owned())
            }
        }
        _ => generate_connection_pool(connection_string.to_owned()),
    };
    let (database_exists, _) =
        check_database_exists(database_name.as_str(), &configuration.database).await;
    if !database_exists {
        panic!("[ERROR]: Database '{}' doesn't exist and the database_migration.migrate property was set to false", database_name.as_str());
    }
    // Raises if failed to bind address
    let bind_address = (
        configuration.application_address.to_owned(),
        configuration.application_port,
    );
    let listener = TcpListener::bind(bind_address).with_context(|| {
        format!(
            "{}::main: Failed to open a TCP Listener on address '{}' and port '{}'.",
            env!("CARGO_PKG_NAME"),
            configuration.application_address.to_owned(),
            configuration.application_port
        )
    })?;
    let mut admin_bind_address = None;
    if configuration.admin_port.is_some() {
        if configuration.admin_address.is_some() {
            admin_bind_address = Some((
                configuration.admin_address.unwrap(),
                configuration.admin_port.unwrap(),
            ));
        } else {
            admin_bind_address = Some((
                configuration.application_address.to_owned(),
                configuration.admin_port.unwrap(),
            ));
        }
    }
    let health_cache_validity_ms: Option<Duration> =
        if configuration.health_cache_validity_ms.is_some() {
            Some(Duration::from_millis(
                configuration.health_cache_validity_ms.unwrap().into(),
            ))
        } else {
            None
        };
    // Run server on TcpListener
    let (server1, server2): (Server, Option<Server>) = run(
        listener,
        postgres_connection,
        admin_bind_address,
        health_cache_validity_ms,
    )
    .unwrap();
    if server2.is_some() {
        future::try_join(server1, server2.unwrap()).await?;
        return Ok(());
    }
    server1.await?;
    Ok(())
}

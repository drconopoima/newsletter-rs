use deadpool_postgres::Pool;
use env_logger::{Builder, Env};
use newsletter_rs::{
    configuration::{get_configuration, ApplicationSettings, DatabaseSettings},
    postgres::{check_database_exists, generate_connection_pool, migrate_database},
    startup::run,
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            println!("[WARNING]: Failed to retrieve a database name from settings, using default value '{}'", database_name);
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
        Some(migration_settings) => {
            if migration_settings.migrate {
                let mut folder = "./migrations".to_owned();
                if let Some(migration_folder) = migration_settings.folder {
                    folder = migration_folder;
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
    let bind_address: (&str, u16) = ("127.0.0.1", configuration.application_port);
    let listener = TcpListener::bind(bind_address)?;
    // env_logger init() to call set_logger. RUST_LOG to customize logging level
    Builder::from_env(Env::default().default_filter_or("info")).init();
    // Run server on TcpListener
    run(listener, postgres_connection)?.await
}

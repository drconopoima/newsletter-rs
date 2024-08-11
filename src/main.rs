use actix_web::dev::Server;
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use futures::future;
use newsletter_rs::{
    censoredstring::CensoredString,
    configuration::{
        get_configuration, DatabaseSettings, MigrationSettings, Settings, SslSettings,
    },
    postgres::{check_database_exists, generate_connection_pool, migrate_database},
    smtp_email_sender::{
        get_smtp_credentials, new_email_builder, new_smtp_relay_mailer, send_email,
    },
    startup::run,
    telemetry,
};
use std::net::TcpListener;
use std::str::FromStr;
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
    let config_file: &str = "main.yaml";
    let configuration: Settings = get_configuration(config_file).unwrap_or_else(|error| {
        panic!(
            "ERROR: Failed to read configuration file \"{}\", {}.",
            config_file, error
        )
    });
    let connection_string = CensoredString::new(
        &configuration.database.connection_string(),
        Some(&configuration.database.connection_string_censored()),
    );
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
    let migration_settings = configuration
        .database
        .migration
        .as_ref()
        .map(|migrationsettings| MigrationSettings {
            migrate: migrationsettings.migrate,
            folder: migrationsettings.folder.to_owned(),
        });
    let database_settings = DatabaseSettings {
        port: configuration.database.port,
        host: configuration.database.host.to_owned(),
        username: configuration.database.username.to_owned(),
        password: CensoredString::from_str(configuration.database.password.as_ref()).unwrap(),
        database: Some(database_name.to_owned()),
        migration: migration_settings,
        ssl: SslSettings {
            tls: configuration.database.ssl.tls,
            cacertificates: configuration.database.ssl.cacertificates.to_owned(),
        },
    };
    let postgres_connection: Pool = match configuration.database.migration {
        Some(ref migration) => {
            if migration.migrate {
                migrate_database(database_settings).await
            } else {
                generate_connection_pool(
                    &connection_string,
                    database_settings.ssl.tls,
                    database_settings.ssl.cacertificates.as_ref(),
                )?
            }
        }
        _ => generate_connection_pool(
            &connection_string,
            database_settings.ssl.tls,
            database_settings.ssl.cacertificates.as_ref(),
        )?,
    };
    let (database_exists, _) =
        check_database_exists(database_name.as_str(), &configuration.database).await;
    if !database_exists {
        panic!("[ERROR]: Database '{}' doesn't exist and the database_migration.migrate property was set to false", database_name.as_str());
    }
    // Raises if failed to bind address
    let bind_address = (
        configuration.application.address.to_owned(),
        configuration.application.port,
    );
    let listener = TcpListener::bind(bind_address).with_context(|| {
        format!(
            "{}::main: Failed to open a TCP Listener on address '{}' and port '{}'.",
            env!("CARGO_PKG_NAME"),
            configuration.application.address.to_owned(),
            configuration.application.port
        )
    })?;
    let mut admin_bind_address = None;
    if let Some(ref admin) = configuration.admin {
        admin_bind_address = Some((admin.address.to_owned(), admin.port));
    }
    let health_cache_validity_ms = configuration
        .application
        .healthcachevalidityms
        .map(|ms| Duration::from_millis(ms.into()));
    let smtp_credentials = get_smtp_credentials(
        &configuration.application.smtp.relay.credentials.username,
        &configuration.application.smtp.relay.credentials.password,
    );
    let smtp_pool = new_smtp_relay_mailer(
        &configuration.application.smtp.relay.address,
        smtp_credentials,
        configuration.application.smtp.relay.port,
    )?;
    tracing::info!(
        "The result of testing connection is: {:?}",
        smtp_pool.test_connection()
    );
    // Err(lettre::transport::smtp::Error { kind: Connection, source: Failure(Ssl(Error { code: ErrorCode(1), cause: Some(Ssl(ErrorStack([Error { code: 167772427, library: \"SSL routines\", function: \"tls_validate_record_header\", reason: \"wrong version number\", file: \"ssl/record/methods/tlsany_meth.c\", line: 80 }]))) }, X509VerifyResult { code: 0, error: \"ok\" })) })"
    /*
    let email_builder = new_email_builder(
        Some(&configuration.application.smtp.name),
        &configuration.application.smtp.from,
        &configuration.application.smtp.reply_to,
    )?;
    send_email(
        Some("Dr. Conopoima"),
        "email@example.com",
        email_builder,
        &smtp_pool,
        "Test Rust SMTP",
        "Hello, this is a test email from Rust!",
    )?;
    */
    // Run server on TcpListener
    let (server1, server2): (Server, Option<Server>) = run(
        listener,
        postgres_connection,
        Some(smtp_pool),
        admin_bind_address,
        health_cache_validity_ms,
    )?;
    if server2.is_some() {
        future::try_join(server1, server2.unwrap()).await?;
        return Ok(());
    }
    server1.await?;
    Ok(())
}

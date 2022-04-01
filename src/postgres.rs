use crate::configuration::{CensoredString, DatabaseSettings};
use actix_web::{body::MessageBody, web::Bytes};
use anyhow::{Context, Error, Result};
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, PoolError, RecyclingMethod};
use md5;
use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use std::fs::{read_dir, File};
use std::io::{BufReader, Read};
use std::str::FromStr;
use tokio_postgres::{NoTls, SimpleQueryMessage};
use tracing::warn;
use uuid::Uuid;

pub fn get_tls_connector(cacertificates: Option<&String>) -> Result<TlsConnector, Error> {
    Ok(if let Some(cert) = cacertificates {
        let mut connector = TlsConnector::builder();
        let vec_certificates: Vec<&str> =
            cert.split_inclusive("-----END CERTIFICATE-----").collect();
        for certificate_string in vec_certificates {
            let cert_string = <&str>::clone(&certificate_string).to_owned();
            if let Ok(certificate) = Certificate::from_pem(
                &cert_string
                    .try_into_bytes()
                    .unwrap_or_else(|_| Bytes::new()),
            ) {
                connector.add_root_certificate(certificate);
            } else {
                warn!(
                    "{}::postgres::get_tls_connector: Failed to create certificate from bytes: {}",
                    env!("CARGO_PKG_NAME"),
                    certificate_string
                );
            };
        }
        connector.build()?
    } else {
        TlsConnector::builder().build()?
    })
}

#[tracing::instrument(name = "Generating database connection pool.")]
pub fn generate_connection_pool(
    postgres_connection_string: CensoredString,
    tls: bool,
    cacertificates: Option<&String>,
) -> Result<Pool, Error> {
    let postgres_configuration =
        tokio_postgres::Config::from_str(&postgres_connection_string).with_context(|| {format!("{}::postgres::generate_connection_pool: Failed to retrieve configuration from connection string '{}'", env!("CARGO_PKG_NAME"), &postgres_connection_string)})?;
    let deadpool_manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Verified,
    };
    if tls {
        let connector: TlsConnector = get_tls_connector(cacertificates)?;
        let connector = MakeTlsConnector::new(connector);
        let deadpool_manager =
            Manager::from_config(postgres_configuration, connector, deadpool_manager_config);
        Ok(Pool::builder(deadpool_manager)
            .max_size(16)
            .build().with_context(|| {format!("{}::postgres::generate_connection_pool: Failed to build TLS connection pool to postgres", env!("CARGO_PKG_NAME"))})?)
    } else {
        let deadpool_manager =
            Manager::from_config(postgres_configuration, NoTls, deadpool_manager_config);
        Ok(Pool::builder(deadpool_manager)
            .max_size(16)
            .build().with_context(|| {format!("{}::postgres::generate_connection_pool: Failed to build NoTLS connection pool to postgres", env!("CARGO_PKG_NAME"))})?)
    }
}

#[tracing::instrument(name = "Getting postgres client from pool.", skip(pool))]
pub async fn get_client(pool: Pool) -> Result<Object, PoolError> {
    pool.get().await
}

#[tracing::instrument(name = "Checking if database exists.")]
pub async fn check_database_exists(
    database_name: &str,
    database_settings: &DatabaseSettings,
) -> (bool, Object) {
    let connection_string_without_database = CensoredString {
        data: database_settings.connection_string_without_database(),
        representation: database_settings.connection_string_without_database_censored(),
    };
    let postgres_pool_without_database: Pool = generate_connection_pool(
        connection_string_without_database,
        database_settings.ssl.tls,
        database_settings.ssl.cacertificates.as_ref(),
    )
    .unwrap();
    let postgres_client = postgres_pool_without_database
        .get()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to get client connection to postgres from pool: \"{}\"",
                error
            )
        });
    let check_database_query = format!(
        "SELECT 1 FROM pg_database WHERE datname = '{}'",
        database_name
    );
    let database_existence_result = &run_simple_query(&postgres_client, &check_database_query)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to retrieve rows to assert database existence with query \"{}\": {}",
                check_database_query, error
            )
        })[0];
    if let SimpleQueryMessage::CommandComplete(number_rows) = database_existence_result {
        if *number_rows == 0 {
            return (false, postgres_client);
        }
    }
    (true, postgres_client)
}

#[tracing::instrument(name = "Creating database.")]
pub async fn create_database(database_settings: &mut DatabaseSettings) -> Result<Pool, Error> {
    let database_name: &str = database_settings.database.as_ref().unwrap().as_str();
    let (exists, postgres_client) = check_database_exists(database_name, database_settings).await;
    if exists {
        database_settings.database = Some(database_name.to_owned());
    } else {
        let create_database_query = format!("CREATE DATABASE \"{}\"", database_name);
        run_simple_query(&postgres_client, &create_database_query)
            .await
            .expect("Failed to create database");
    }
    let connection_string = CensoredString {
        data: database_settings.connection_string(),
        representation: database_settings.connection_string_censored(),
    };
    generate_connection_pool(
        connection_string,
        database_settings.ssl.tls,
        database_settings.ssl.cacertificates.as_ref(),
    )
}

#[tracing::instrument(name = "Migrating Database.")]
pub async fn migrate_database(mut database_settings: DatabaseSettings) -> Pool {
    // Ensure database creation
    let postgres_pool = create_database(&mut database_settings).await.unwrap();
    let postgres_client = postgres_pool
        .get()
        .await
        .expect("Failed to generate client connection to postgres from pool");
    let mut path_entries: Vec<_> = read_dir(database_settings.migration.unwrap().folder)
        .expect("Failed to read database migrations directory")
        .map(|r| r.unwrap())
        .collect();
    path_entries.sort_by_key(|dir| dir.path());
    let mut migration_script_paths: Vec<String> = Vec::new();
    for path in path_entries {
        migration_script_paths.push(path.path().display().to_string());
    }
    create_migrations_table(&postgres_client).await;
    let check_migration_statement = match postgres_client
        .prepare_cached(
            r#"
                    SELECT 1 FROM _initialization_migrations WHERE md5_hash=$1
                    "#,
        )
        .await
    {
        Ok(statement) => statement,
        Err(error) => {
            panic!(
                "Failed to prepare cached check migration statement, {}",
                error
            );
        }
    };
    let insert_migration_statement = match postgres_client
        .prepare_cached(
            r#"
                    INSERT into _initialization_migrations (filename, md5_hash)
                        VALUES ($1, $2)
                    "#,
        )
        .await
    {
        Ok(statement) => statement,
        Err(error) => {
            panic!(
                "Failed to prepare cached insert migration statement, {}",
                error
            );
        }
    };
    for migration_script_path in migration_script_paths {
        let migration_file = File::open(&migration_script_path).unwrap();
        let mut reader = BufReader::new(migration_file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap_or_else(|error| {
            panic!(
                "Failed to read contents of file {}: {}",
                &migration_script_path, error
            )
        });
        let migration_file_contents = String::from_utf8_lossy(&buffer).to_string();
        let md5_script_digest = md5::compute(&migration_file_contents);
        let md5_script_string = format!("{:x}", md5_script_digest);
        let script_md5_uuid = Uuid::parse_str(&md5_script_string).unwrap();
        let migration_run_result = &postgres_client
            .query_opt(&check_migration_statement, &[&script_md5_uuid])
            .await
            .unwrap();
        if migration_run_result.is_none() {
            run_simple_query(&postgres_client, &migration_file_contents)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to perform query migration of file {}: {}",
                        &migration_script_path, error
                    )
                });
            let file_path = std::path::Path::new(&migration_script_path);
            let filename_script: &str = file_path.file_name().unwrap().to_str().unwrap();
            postgres_client
                .query(
                    &insert_migration_statement,
                    &[&filename_script, &script_md5_uuid],
                )
                .await
                .unwrap_or_else(|error| panic!("Failed to insert migration: {}", error));
        }
    }
    postgres_pool
}

#[tracing::instrument(name = "Creating migrations table.", skip(postgres_client))]
pub async fn create_migrations_table(postgres_client: &Object) {
    let migration_table_name = "_initialization_migrations";
    let create_table_statement = format!(
        "CREATE TABLE IF NOT EXISTS {}(
        version SERIAL PRIMARY KEY,
        filename TEXT NOT NULL,
        installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
        md5_hash UUID NOT NULL
    );",
        &migration_table_name
    );
    let _ = run_simple_query(postgres_client, &create_table_statement)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to create table \"{}\" with query \"{}\": {}",
                migration_table_name, create_table_statement, error
            )
        })[0];
}

#[tracing::instrument(name = "Running simple query.", skip(postgres_client))]
pub async fn run_simple_query(
    postgres_client: &Object,
    query_statement: &str,
) -> Result<Vec<SimpleQueryMessage>, tokio_postgres::Error> {
    postgres_client.simple_query(query_statement).await
}

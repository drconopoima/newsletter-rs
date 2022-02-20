use crate::configuration::DatabaseSettings;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use md5;
use std::fs::{read_dir, File};
use std::io::{BufReader, Read};
use std::str::FromStr;
use tokio_postgres::{Error, NoTls, SimpleQueryMessage};
use uuid::Uuid;

pub fn generate_connection_pool(postgres_connection_string: String) -> Pool {
    let postgres_configuration =
        tokio_postgres::Config::from_str(&postgres_connection_string).unwrap();
    let deadpool_manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Verified,
    };
    let deadpool_manager =
        Manager::from_config(postgres_configuration, NoTls, deadpool_manager_config);
    Pool::builder(deadpool_manager)
        .max_size(16)
        .build()
        .unwrap()
}

pub async fn get_client(pool: Pool) -> Object {
    pool.get().await.unwrap()
}

pub async fn check_database_exists(
    database_name: &str,
    database_settings: &DatabaseSettings,
) -> (bool, Object) {
    let connection_string_without_database = database_settings.connection_string_without_database();
    let postgres_pool_without_database: Pool =
        generate_connection_pool(connection_string_without_database);
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
    let database_existence_result = &postgres_client
        .simple_query(&check_database_query)
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

pub async fn create_database(database_settings: &mut DatabaseSettings) -> Pool {
    let database_name: &str = database_settings.database.as_ref().unwrap().as_str();
    let (exists, postgres_client) = check_database_exists(database_name, database_settings).await;
    if exists {
        database_settings.database = Some(database_name.to_string());
    } else {
        let create_database_query = format!("CREATE DATABASE \"{}\"", database_name);
        run_simple_query(&postgres_client, &create_database_query)
            .await
            .expect("Failed to create database");
    }
    let connection_string = database_settings.connection_string();
    generate_connection_pool(connection_string)
}

pub async fn migrate_database(
    mut database_settings: DatabaseSettings,
    migration_folder: String,
) -> Pool {
    // Ensure database creation
    let postgres_pool = create_database(&mut database_settings).await;
    let postgres_client = postgres_pool
        .get()
        .await
        .expect("Failed to generate client connection to postgres from pool");
    let mut path_entries: Vec<_> = read_dir(migration_folder)
        .expect("Failed to read database migrations directory")
        .map(|r| r.unwrap())
        .collect();
    path_entries.sort_by_key(|dir| dir.path());
    let mut migration_script_paths: Vec<String> = Vec::new();
    for path in path_entries {
        migration_script_paths.push(path.path().display().to_string());
    }
    create_migrations_table(&postgres_client).await;
    let statement = match postgres_client
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
            panic!("Failed to prepare cached insert migration: {}", error);
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
        run_simple_query(&postgres_client, &migration_file_contents)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to perform query migration of file {}: {}",
                    &migration_script_path, error
                )
            });
        let md5_script_digest = md5::compute(&migration_file_contents);
        let md5_script_string = format!("{:x}", md5_script_digest);
        let script_md5_uuid = Uuid::parse_str(&md5_script_string).unwrap();
        let file_path = std::path::Path::new(&migration_script_path);
        let filename_script: &str = file_path.file_name().unwrap().to_str().unwrap();
        postgres_client
            .query(&statement, &[&filename_script, &script_md5_uuid])
            .await
            .unwrap_or_else(|error| panic!("Failed to insert migration: {}", error));
    }
    postgres_pool
}

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

pub async fn run_simple_query(
    postgres_client: &Object,
    query_statement: &str,
) -> Result<Vec<SimpleQueryMessage>, Error> {
    postgres_client.simple_query(query_statement).await
}

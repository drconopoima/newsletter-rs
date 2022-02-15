use crate::configuration::DatabaseSettings;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use std::fs::{read_dir, File};
use std::io::{BufReader, Read};
use std::str::FromStr;
use tokio_postgres::{NoTls, SimpleQueryMessage};

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
        postgres_client
            .simple_query(&create_database_query)
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
        postgres_client
            .simple_query(&migration_file_contents)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "Failed to perform query migration of file {}: {}",
                    &migration_script_path, error
                )
            });
    }
    postgres_pool
}

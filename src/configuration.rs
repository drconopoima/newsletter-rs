use config::{Config, File, FileFormat};
use std::fmt;

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub database: DatabaseSettings,
    pub application_address: String,
    pub application_port: u16,
    pub health_cache_validity_ms: Option<u32>,
    pub admin_address: Option<String>,
    pub admin_port: Option<u16>,
    pub database_migration: Option<MigrationSettings>,
}

#[derive(serde::Deserialize)]
pub struct MigrationSettings {
    pub migrate: bool,
    pub folder: Option<String>,
}

#[derive(serde::Deserialize)]

pub struct DatabaseSettings {
    pub port: u16,
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        if self.database.is_none() {
            format!(
                "postgresql://{}:{}@{}:{}",
                self.username, self.password, self.host, self.port
            )
        } else {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.username,
                self.password,
                self.host,
                self.port,
                self.database.as_ref().unwrap()
            )
        }
    }
    pub fn connection_string_without_database(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
// Read top-level configuration file with extension YAML...
pub fn get_configuration(filename: &str) -> Result<ApplicationSettings, config::ConfigError> {
    // Initialize configuration reader
    let builder = Config::builder()
        .add_source(File::new(filename, FileFormat::Yaml))
        .build()
        .unwrap();
    // Convert into Result<Settings, ConfigError>
    builder.try_deserialize::<ApplicationSettings>()
}

impl fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("username", &self.username)
            .field("database", &self.database)
            .finish()
    }
}

use anyhow::{Context, Error, Result};
use config::{Config, Environment, File, FileFormat};
use serde_aux::field_attributes::{
    deserialize_number_from_string, deserialize_option_number_from_string,
};
use std::fmt;
use tracing::info;

pub static CONFIGURATION_SUBDIRECTORY: &str = "configuration";

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub admin: Option<AdminSettings>,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub address: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub healthcachevalidityms: Option<u32>,
}

#[derive(serde::Deserialize)]
pub struct AdminSettings {
    pub address: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct MigrationSettings {
    pub migrate: bool,
    pub folder: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub migration: Option<MigrationSettings>,
    pub ssl: SslSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct SslSettings {
    pub tls: bool,
    pub cacertificates: Option<String>,
}

impl fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("username", &self.username)
            .field("password", &self.password)
            .field("database", &self.database)
            .finish()
    }
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        if self.database.is_none() {
            format!(
                "postgresql://{}:{}@{}:{}/",
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
            "postgresql://{}:{}@{}:{}/",
            self.username, self.password, self.host, self.port
        )
    }
    pub fn connection_string_censored(&self) -> String {
        if self.database.is_none() {
            format!(
                "postgresql://{}:{}@{}:{}/",
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
    pub fn connection_string_without_database_censored(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/",
            self.username, self.password, self.host, self.port
        )
    }
}

// Read top-level configuration file with extension YAML...
pub fn get_configuration(filename: &str) -> Result<Settings, Error> {
    let environment = std::env::var("APP__ENVIRONMENT").unwrap_or_else(|_| "local".to_owned());
    // Initialize configuration reader
    let default_configuration_file = &*format!("{}/{}", CONFIGURATION_SUBDIRECTORY, filename);
    let environment_configuration_file =
        &*format!("{}/{}", CONFIGURATION_SUBDIRECTORY, environment);
    let builder = Config::builder()
        .add_source(File::new(default_configuration_file, FileFormat::Yaml))
        .add_source(File::new(environment_configuration_file,FileFormat::Yaml))
        .add_source(Environment::with_prefix("APP_").try_parsing(true).separator("_"))
        .build()
        .with_context(|| {
            format!(
                "{}::configuration::get_configuration: Failed to build configuration from sources: '{}' and '{}'",
                env!("CARGO_PKG_NAME"),
                default_configuration_file,
                environment_configuration_file
            )
        })?;
    info!("Successfully built configuration: '{:?}'", builder);
    // Convert into Result<Settings, ConfigError>
    builder.try_deserialize::<Settings>().with_context(|| {
        format!(
            "{}::configuration::get_configuration: Failed to deserialize configuration",
            env!("CARGO_PKG_NAME")
        )
    })
}

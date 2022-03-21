use config::{Config, File, FileFormat};
use std::fmt;
use std::ops::{Deref, DerefMut};

pub static CENSOR_STRING: &str = "***REMOVED***";

#[derive(serde::Deserialize)]
pub struct CensoredString {
    pub data: String,
    pub representation: String,
}
// Antipattern Deref polymorphism to emulate inheritance. Read https://github.com/rust-unofficial/patterns/blob/main/anti_patterns/deref.md
impl Deref for CensoredString {
    type Target = String;
    fn deref(&self) -> &String {
        &self.data
    }
}
// Deref coercion for DerefMut to emulate inheritance. Read https://doc.rust-lang.org/std/ops/trait.DerefMut.html
impl DerefMut for CensoredString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl fmt::Debug for CensoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.representation, f)
    }
}
impl fmt::Display for CensoredString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.representation, f)
    }
}

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
impl fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("username", &self.username)
            .field("password", &CENSOR_STRING)
            .field("database", &self.database)
            .finish()
    }
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
    pub fn connection_string_censored(&self) -> String {
        if self.database.is_none() {
            format!(
                "postgresql://{}:{}@{}:{}",
                self.username, &CENSOR_STRING, self.host, self.port
            )
        } else {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.username,
                &CENSOR_STRING,
                self.host,
                self.port,
                self.database.as_ref().unwrap()
            )
        }
    }
    pub fn connection_string_without_database_censored(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}",
            self.username, &CENSOR_STRING, self.host, self.port
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

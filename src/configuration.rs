use anyhow::{Context, Error, Result};
use config::{Config, Environment, File, FileFormat};
use serde_aux::field_attributes::{
    deserialize_number_from_string, deserialize_option_number_from_string,
};
use std::fmt;
use std::any::type_name;
use std::str::FromStr;
use tracing::info;
use zeroize::Zeroize;
use serde::Serialize;

pub static CENSOR_STRING: &str = "***REMOVED***";
pub static CONFIGURATION_SUBDIRECTORY: &str = "configuration";

/// Marker trait for secrets which are allowed to be cloned
pub trait CloneableSecret: Clone + Zeroize {}

/// Implement `CloneableSecret` on arrays of types that impl `Clone` and
/// `Zeroize`.
macro_rules! impl_cloneable_secret_for_array {
    ($($size:expr),+) => {
        $(
            impl<T: Clone + Zeroize> CloneableSecret for [T; $size] {}
        )+
     };
}

impl_cloneable_secret_for_array!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64
);

pub trait SerializableSecret: Serialize {}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Secret<T>
where
    T: Zeroize + Clone + de::DeserializeOwned + Sized,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Secret::new)
    }
}

#[cfg(feature = "serde")]
impl<T> Serialize for Secret<T>
where
    T: Zeroize + SerializableSecret + Serialize + Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.expose_secret().serialize(serializer)
    }
}

/// Expose a reference to an inner secret
pub trait ExposeSecret<S> {
    /// Expose secret: this is the only method providing access to a secret.
    fn expose_secret(&self) -> &S;
}

/// Debugging trait which is specialized for handling secret values
pub trait DebugSecret {
    /// Format information about the secret's type.
    ///
    /// This can be thought of as an equivalent to [`Debug::fmt`], but one
    /// which by design does not permit access to the secret value.
    fn debug_secret(f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("[REDACTED ")?;
        f.write_str(type_name::<Self>())?;
        f.write_str("]")
    }
}

#[derive(serde::Deserialize)]
pub struct Secret<S>
where
    S: Zeroize,
{
    /// Inner secret value
    inner_secret: S,
}

impl<S> Secret<S>
where
    S: Zeroize,
{
    /// Take ownership of a secret value
    pub fn new(secret: S) -> Self {
        Secret {
            inner_secret: secret,
        }
    }
}

impl<S> ExposeSecret<S> for Secret<S>
where
    S: Zeroize,
{
    fn expose_secret(&self) -> &S {
        &self.inner_secret
    }
}

impl<S> From<S> for Secret<S>
where
    S: Zeroize,
{
    fn from(secret: S) -> Self {
        Self::new(secret)
    }
}

impl<S> Clone for Secret<S>
where
    S: CloneableSecret,
{
    fn clone(&self) -> Self {
        Secret {
            inner_secret: self.inner_secret.clone(),
        }
    }
}

impl<S> fmt::Debug for Secret<S>
where
    S: Zeroize + DebugSecret,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(")?;
        S::debug_secret(f)?;
        f.write_str(")")
    }
}

impl<S> Drop for Secret<S>
where
    S: Zeroize,
{
    fn drop(&mut self) {
        // Zero the secret out from memory
        self.inner_secret.zeroize();
    }
}

/// Secret strings
pub type SecretString = Secret<String>;

impl DebugSecret for String {}
impl CloneableSecret for String {}

impl FromStr for SecretString {
    type Err = core::convert::Infallible;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(SecretString::new(src.to_string()))
    }
}

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
    pub password: SecretString,
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
                self.username, self.password.expose_secret(), self.host, self.port
            )
        } else {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.database.as_ref().unwrap()
            )
        }
    }
    pub fn connection_string_without_database(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
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

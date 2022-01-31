use config;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub port: u16,
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}
// Read top-level configuration file with compatible extension YAML,JSON...
pub fn get_configuration(filename: &str) -> Result<Settings, config::ConfigError> {
    // Initialize configuration reader
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name(filename))?;
    // Convert into Result<Settings, ConfigError>
    settings.try_into()
}

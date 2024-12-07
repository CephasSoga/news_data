use serde::Deserialize;
use config::{builder::DefaultState, ConfigBuilder, ConfigError, File};


#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub alphavantage: String,
    pub marketaux: String
}

#[derive(Debug, Deserialize)]
pub struct ValueConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
}
impl ValueConfig {
    pub fn new() -> Result<Self, ConfigError> {
    // Builder
    let mut builder: ConfigBuilder<DefaultState> = ConfigBuilder::default(); // Use default() instead of new()

    // Start off by merging in the "default" configuration file
    builder = builder.add_source(File::with_name("config")); // Example of adding a file source


    // Build the configuration
    let config = builder.build()
        .map_err(|e| {
            return ConfigError::FileParse { uri: Some(e.to_string()), cause: Box::new(e) }
        })?;

    // Deserialize the configuration into our Config struct
    // return it
    config.try_deserialize()

    }
}

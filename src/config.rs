use std::fmt;
use std::hash::Hash;

use serde::Deserialize;
use config::{builder::DefaultState, ConfigBuilder, ConfigError, File};


#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub name: String,
    pub database_name: String,
    pub collection_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Hash, Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiConfig {
    pub alphavantage: String,
    pub marketaux: String,
    pub fmp: String
}

#[derive(Debug, Clone, Hash, Deserialize)]
pub struct RequestArgs {
    pub delay_secs: i64
}
#[derive(Clone, Debug, Deserialize)]
pub struct TaskArgs {
    pub base_delay_ms: u32,
    pub max_delay_ms: u32,
    pub max_retries: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ValueConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
    pub request: RequestArgs,
    pub task: TaskArgs,
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

impl fmt::Display for ValueConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Format the fields of ValueConfig as needed
        write!(f, "MarketAux API Key: {}*****, AlphavantageAPI: {}*****", 
               self.api.marketaux.get(..4).unwrap_or(""), // Safely get the first 4 characters
               self.api.alphavantage.get(..4).unwrap_or("")) // Replace with actual fields
    }
}

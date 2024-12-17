//! This module handles fetching news data from MarketAux and AlphaVantage APIs,
//! caching the results, and inserting them into a MongoDB database.

#[allow(dead_code)]


use std::fmt;

use cached::TimedCache;
use cached::proc_macro::cached;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::time::{sleep, Duration};

use alphavantage::AlphaVantageApiResponse;
use marketaux::MarketAuxResponse;

use crate::utils::{time_rfc3339_opts, now, generate_random_key};

pub mod marketaux;
pub mod alphavantage;
pub mod db;
pub mod config;
pub mod utils;


/// Custom error type for fetching news data.
#[derive(Debug, Clone)]
pub struct FetchNewsError {
    pub message: String,
}

impl std::error::Error for FetchNewsError {}

impl fmt::Display for FetchNewsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Struct representing the result of fetching news data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewsResult {
    hash_key: String,
    marketaux: MarketAuxResponse,
    alphavantage: AlphaVantageApiResponse,
    from: String,
    to: String,
    time_range: u64,
    marketaux_data_len: u64,
    alphavantage_data_len: u64
}
impl NewsResult {
    /// Checks if two NewsResult instances are equal based on hash_key, from, and to fields.
    pub fn eq(&self, other: &Self) -> bool {
        self.hash_key == other.hash_key && 
        self.from == other.from &&
        self.to == other.to
    }

    /// Converts the NewsResult instance to a JSON value.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("Failed to convert to JSON value") 
    }
}

/// Fetches news data from MarketAux and AlphaVantage APIs, with caching.
#[cached(
    type = "TimedCache<String, Result<NewsResult, FetchNewsError>>",
    create = "{ TimedCache::with_lifespan(600) }", // Cache lifespan of 10 minutes
    convert = r#"{ format!("{:?}", config) }"#
)]
async fn fetch_news_data(config: &config::ValueConfig) -> Result<NewsResult, FetchNewsError> {

    let marketaux_data = marketaux::run(config)
        .await
        .map_err(|e| FetchNewsError { message: format!("MarketAux error: {}", e) })?;
    
    let alphavantage_data = alphavantage::run(config)
        .await
        .map_err(|e| FetchNewsError { message: format!("AlphaVantage error: {}", e) })?;

    Ok(NewsResult {
        hash_key: generate_random_key(8),
        marketaux: marketaux_data.clone(),
        alphavantage: alphavantage_data.clone(),
        from: time_rfc3339_opts(config.request.delay_secs),
        to: now(),
        time_range: config.request.delay_secs as u64,
        marketaux_data_len: marketaux_data.data.len() as u64,
        alphavantage_data_len: alphavantage_data.feed.len() as u64,
    })
}

/// Main function that reads the config, initializes the database client, 
/// fetches news data in a loop, and inserts it into the database.
#[tokio::main]
async fn main() -> Result<(), FetchNewsError> {
    println!("Reading config file...");
    let value_config = config::ValueConfig::new().expect("Failed to read config file");

    println!("Creating databse client...");
    let db_client = db::ClientManager::new(&value_config).await.map_err(
        |e| {e}
    ).unwrap();

    println!("Getting ready...");
    let db_ops = db::DatabaseOps::new(
        db_client.get_client(), 
        &value_config.database.database_name, 
        &value_config.database.collection_name);

    println!("Fetching data....");
    loop {
        match fetch_news_data(&value_config).await {
            Ok(data) => {
                println!(
                "GET request yielded: {} results\n
                Hash key: {} \n",
                data.marketaux_data_len + data.alphavantage_data_len,
                data.hash_key );

                println!("Inserting into database...");
                let doc = db_ops.convert_to_document(data.to_json())
                    .map_err(|e| println!("Error converting NewsResult to bson::Document: {}", e))
                    .unwrap();

                let _ = db_ops.insert_one(doc).await
                    .map_err(|e| println!("Error inserting document: {}", e))
                    .unwrap();

                println!("Done.");
            },
            Err(e) => eprintln!("Error fetching news data: {}", e),
        }

        // Sleep to throttle requests
        sleep(Duration::from_secs(value_config.request.delay_secs as u64)).await;
    }
}

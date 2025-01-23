//! This module handles fetching news data from MarketAux and AlphaVantage APIs,
//! caching the results, and inserting them into a MongoDB database.

#![allow(dead_code)]
#![allow(unused_imports)]


use std::fmt;
use std::sync::Arc;

use cache::SharedLockedCache;
use cached::TimedCache;
use cached::proc_macro::cached;
use request::HTTPClient;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use tracing::{trace, info, error, warn, debug};

use alphavantage::AlphaVantageApiResponse;
use marketaux::MarketAuxResponse;

use crate::utils::{time_rfc3339_opts, now, generate_random_key};
use crate::logging::setup_logger;
use crate::fmp::FMPClient;
use crate::config::ValueConfig;
use marketaux::{ALL_NEWS_ENDPOINT, SIMILAR_NEWS_ENDPOINT, NEWS_BY_UUID};

pub mod fmp;
pub mod marketaux;
pub mod alphavantage;
pub mod db;
pub mod config;
pub mod utils;
pub mod logging;
pub mod options;
pub mod request;
pub mod server_types;
pub mod cache;


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
async fn fetch_news_data(req_client: Arc<Client>, config: Arc<ValueConfig>) -> Result<NewsResult, FetchNewsError> {

    let marketaux_data = marketaux::run(
            ALL_NEWS_ENDPOINT, 
            req_client.clone(), 
            config.clone()
        ).await
        .inspect(|data| info!("Successfully fetched from marketaux. | Meta :{:?}", data.meta))
        .map_err(|e| FetchNewsError { message: format!("MarketAux error: {}", e)})?;
    
    let alphavantage_data = alphavantage::run(
            req_client.clone(), 
            config.clone()
        ).await
        .inspect(|data| info!("Successfully fetched data from Alphavantage. | Meta: {:?}", data.items))
        .map_err(|e| FetchNewsError { message: format!("AlphaVantage error: {}", e)})?;

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
    // Initialize tracing
    setup_logger("trace");

    info!("Reading config file & Preparing components...");
    let value_config = Arc::new(config::ValueConfig::new().expect("Failed to read config file"));
    let req_client = Arc::new(Client::new());

    info!("Creating databse client...");
    let db_client = db::ClientManager::new(&value_config).await.map_err(
        |e| {e}
    ).unwrap();

    info!("Getting ready...");
    let db_ops = db::DatabaseOps::new(
        db_client.get_client(), 
        &value_config.database.database_name, 
        &value_config.database.collection_name);

    info!("Fetching data....");
    loop {
        match fetch_news_data(req_client.clone(), value_config.clone()).await {
            Ok(data) => {
                trace!(
                "GET request yielded: {} results | Hash key: {} \n",
                data.marketaux_data_len + data.alphavantage_data_len,
                data.hash_key );

                info!("Inserting into database...");
                let doc = db_ops.convert_to_document(data.to_json())
                    .map_err(|e| error!("Error converting NewsResult to bson::Document: {}", e))
                    .unwrap();

                let _ = db_ops.insert_one(doc).await
                    .map_err(|e| error!("Error inserting document: {}", e))
                    .unwrap();

                info!("Done.");
            },
            Err(e) => error!("Error fetching news data: {}", e),
        }

        // Sleep to throttle requests
        info!("Next fetch in {} seconds", value_config.request.delay_secs);
        sleep(Duration::from_secs(value_config.request.delay_secs as u64)).await;
    }
}


#[tokio::main]
async fn main_() {
    // Initialize tracing
    setup_logger("trace");

    // Fetch news data
    info!("Fetching news...");
    let args = json!({
        "function": "stock news"
    });

    info!("Initializing cache...");
    let cache = Arc::new(Mutex::new(SharedLockedCache::new(100 as usize)));

    info!("Initializing HTTP client...");
    let http_client = Arc::new(HTTPClient::new().expect("Failed to initialize HTTP client."));

    info!("Reading configurations...");
    let config = Arc::new(ValueConfig::new().expect("Configurations were not properly parsed."));

    info!("Creating FMP client...");
    let fmp_client = FMPClient::new(http_client, cache, config);

    info!("Now fetching news data...");
    let response = fmp_client.poll(args).await;
    debug!("Request yielded a Response {:?}: ", response.is_ok());
}
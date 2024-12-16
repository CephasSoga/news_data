#[allow(dead_code)]

use std::fmt;

use cached::TimedCache;
use cached::proc_macro::cached;
use tokio::time::{sleep, Duration};

use alphavantage::AlphaVantageApiResponse;
use marketaux::MarketAuxResponse;

pub mod marketaux;
pub mod alphavantage;
pub mod db;
pub mod config;
pub mod utils;


// Custom Error Type
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

// Your NewsResult struct
#[derive(Clone, Debug, PartialEq)]
pub struct NewsResult {
    marketaux: MarketAuxResponse,
    alphavantage: AlphaVantageApiResponse,
}

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
        marketaux: marketaux_data,
        alphavantage: alphavantage_data,
    })
}

#[tokio::main]
async fn main() -> Result<(), FetchNewsError> {
    println!("Reading config file...");
    let value_config = config::ValueConfig::new().expect("Failed to read config file");

    println!("Fetching data....");
    loop {
        match fetch_news_data(&value_config).await {
            Ok(_data) => println!("Done."),
            Err(e) => eprintln!("Error fetching news data: {}", e),
        }

        // Sleep to throttle requests
        sleep(Duration::from_secs(value_config.request.delay_secs as u64)).await;
    }
}

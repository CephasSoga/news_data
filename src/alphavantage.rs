//! ## A Rust wrapper of the [Alpha Vanatge API](https://www.alphavantage.co).
//! 
//! This API returns live and historical market news & sentiment data from a large & growing selection of premier news 
//! outlets around the world, covering stocks, cryptocurrencies, forex, and a wide range of topics such as fiscal policy, 
//! mergers & acquisitions, IPOs, etc. 
//! This API, combined with our core stock API, fundamental data, and technical indicator APIs, 
//! can provide you with a 360-degree view of the financial market and the broader economy.
//! 
//! ## Reference:
//! 
//! [official Alpha Vantage Documentation](https://www.alphavantage.co/documentation/).

#[allow(dead_code)]
#[allow(unused_imports)]

use std::fmt;
use std::time::Duration;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use mongodb::bson::de;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_value};
use reqwest::{Client, Response, StatusCode};
use tracing::{debug, error, info, warn};
use twitter_v2::oauth2::helpers::variant_name;
use tokio::sync::Mutex;

use crate::cache::SharedLockedCache;
use crate::config::ValueConfig;
use crate::utils::{get_resp_value_from_cache_or_fetch, time_yyyy_mmdd_thhmm};
use crate::options::FetchType;
use crate::errors::{AbstractApiError, ApiError};
use crate::options::AVQueryParams as QueryParams;


const BASE_URL: &str = "https://www.alphavantage.co/query";
pub const BASE_FUNCTION: &str = "NEWS_SENTIMENT";
const FETCH_TYPE_KEY_MAP: &str = "fetch_type";


#[derive(Clone, Debug, Serialize, Deserialize)]

/// Wrapper of the Alpha Vantage API response.
/// 
/// [See example here](https://www.alphavantage.co/query?function=NEWS_SENTIMENT&tickers=AAPL&apikey=demo).
pub struct AlphaVantageApiResponse {
    pub items: Option<String>,
    pub sentiment_score_definition: Option<String>,
    pub relevance_score_definition: Option<String>,
    pub feed: Vec<FeedItem>,
}
impl AlphaVantageApiResponse {
    /// Constructs a `AlphaVantageApiResponse` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        from_str(json)
    }

    /// Serializes the `AlphaVantageApiResponse` to a JSON string.
    pub fn to_json(&self) -> Result<Value, ApiError> {
        to_value(self).map_err(|err| ApiError::JsonParseError { message: err.to_string() })
    }

    /// Constructs a `AlphaVantageApiResponse` from a HashMap.
    pub fn from_hashmap(map: HashMap<String, Value>) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(&map)?;
        Self::from_json(&json)
    }
}
impl Hash for AlphaVantageApiResponse {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the relevant fields of MarketAuxResponse
        // Example: state.write(self.some_field.hash());
        self.feed.hash(state)
    }
}

impl PartialEq for AlphaVantageApiResponse {
    fn eq(&self, other: &Self) -> bool {
        // Compare relevant fields of AlphaVantageApiResponse
        // Example:
        self.items == other.items && self.feed == other.feed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedItem {
    pub title: Option<String>,
    pub url: Option<String>,
    pub time_published: Option<String>,
    pub authors: Vec<String>,
    pub summary: Option<String>,
    pub banner_image: Option<String>,
    pub source: Option<String>,
    pub category_within_source: Option<String>,
    pub source_domain: Option<String>,
    pub topics: Vec<Topic>,
    pub overall_sentiment_score: f64,
    pub overall_sentiment_label: Option<String>,
    pub ticker_sentiment: Vec<TickerSentiment>,
}
impl Hash for FeedItem{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.url.hash(state);
    }
}
impl PartialEq for FeedItem {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title &&
        self.url == other.url &&
        self.time_published == other.time_published &&
        self.authors == other.authors
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub topic: Option<String>,
    pub relevance_score: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickerSentiment {
    pub ticker: Option<String>,
    pub relevance_score: Option<String>,
    pub ticker_sentiment_score: Option<String>,
    pub ticker_sentiment_label: Option<String>,
}

pub struct AlphaVantageApiClient {
    client: Arc<Client>,
    cache: Arc<Mutex<SharedLockedCache>>,
    config: Arc<ValueConfig>,
}
impl AlphaVantageApiClient {
        pub fn new(client: Arc<Client>, cache: Arc<Mutex<SharedLockedCache>>, config: Arc<ValueConfig>) -> Self {
        Self {client, cache, config}
    }

    async fn get(
        &self,
        fetch_type: &FetchType,
        endpoint: &str,
        query_params: QueryParams   
    ) -> Result<Value, ApiError> {
        match fetch_type {
            FetchType::AlphaVantage=> {
                let key = format!("{}_{}_{:?}", variant_name(&fetch_type), endpoint, &query_params);
                get_resp_value_from_cache_or_fetch(
                    &self.cache, 
                    &key, 
                    || async{self.get_(endpoint, query_params).await},
                    self.config.task.cache_ttl).await.
                map_err(|e| { 
                    warn!("AlphaVantage client encountered an error during GET request.");
                    e
                })
            },
             _ => return Err(ApiError::RequestError{
                message: format!("Unsupported task: {:?}", &fetch_type), 
                status: None, 
                headers: None, 
                body:None})
        }
    }

    pub async fn get_(
        &self, 
        url: &str, 
        query_params: QueryParams
    ) -> Result<Value, ApiError> {
        // Send GET request
        let response = self
            .client
            .get(url)
            .query(&query_params)
            .send()
            .await.map_err(|e| {
                warn!("AlphaVantage client encountered an error during GET request.");
                // Check if the error is a network error
                if e.is_timeout() || e.is_connect() {
                    ApiError::NetworkError {
                        message: e.to_string(),
                        status: Some(StatusCode::REQUEST_TIMEOUT), //Error: 408 - substitutes to `None`: normaly error is not received here, as the rea did not even go through,
                        headers: None,
                        body: None,
                    }
                } else {
                    ApiError::RequestError{
                        message: e.to_string(),
                        status: Some(StatusCode::BAD_REQUEST),  // Error 400
                        headers: None,
                        body: None
                    }
                }
            })?; // Handle request error

        // Check for rate limit error in response
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let error = self.parse_resp_error(
                "Rate limit exceeded.".to_string(),
                response,
                AbstractApiError::RateLimitError,
            ).await;
            return Err(error);
        } else if response.status().is_server_error() {
            let error = self.parse_resp_error(
                "Internal server error.".to_string(),
                response,
                AbstractApiError::ServerError,
            ).await;
            return Err(error);
        }
        else if response.status() != reqwest::StatusCode::OK {
            let error = self.parse_resp_error(
                "Unhandled error.".to_string(),
                response,
                AbstractApiError::UnhandledError,
            ).await;
            return Err(error);
        }
        
        // # Attempt to parse the JSON response.
        // ** The following lines can have performance implications, especially if the response body is large. 
        // ** This is because it reads the entire response body into memory as a String, which can be inefficient for large payloads.
        // ** If the API changes in the future, uncomment these lines to investigate the parsing errors.
        //: let response_text = response.text().await.unwrap_or_else(|_| String::from("Failed to read body"));
        //: let response_json: AlphaVantageApiResponse = serde_json::from_str(&response_text)
        //:    .map_err(|e| {
        //:        eprintln!("Raw response body: {}", response_text);
        //:        ApiError::JsonParseError { message: e.to_string() }
        //:    })?; // Handle JSON parsing error

        // Attempt to parse the JSON response directly.
        // Also the only place the Response super-struct `AlphavantageApiResponse` is Actually used.
        // For data integrity reasons.
        let response_json: AlphaVantageApiResponse = response.json().await.map_err(|e| {
            error!("Failed to read body: {:?}", e);
            ApiError::JsonParseError { message: e.to_string() }
        })?; // Handle JSON parsing error
        // Bact to Value.
        response_json.to_json()
    }

    /// Parses the response error from the Alpha Vantage API and constructs an appropriate `ApiError`.
    async fn parse_resp_error(&self, message: String, response: Response, abstract_error_type: AbstractApiError) -> ApiError {
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await.unwrap_or_else(|_| String::from("Failed to read body"));

        match abstract_error_type {
            AbstractApiError::RateLimitError  => {
                ApiError::RateLimitError {
                    message,
                    status: Some(status),
                    headers: Some(headers),
                    body: Some(body),
                }
            },

            AbstractApiError::NetworkError => {
                ApiError::NetworkError {
                    message,
                    status: Some(status),
                    headers: Some(headers),
                    body: Some(body),
                }
            },

            AbstractApiError::ServerError => {
                ApiError::ServerError {
                    message,
                    status: Some(status),
                    headers: Some(headers),
                    body: Some(body),
                }
            },
            AbstractApiError::UnhandledError => {
                ApiError::UnhandledError {
                    message,
                    status: Some(status),
                    headers: Some(headers),
                    body: Some(body),
                }
            },
            _ => {
                panic!("Error type not supported! Consider Extending the `ApiError` enum if your use case requires a more granular error handling.")
            },
        }
    }

    fn insert_apikey_and_function(&self, value: Arc<Value>) -> Value{
        let mut value = Arc::try_unwrap(value).unwrap_or_else(|v| (*v).clone());
        if let Value::Object(ref mut map) = value {
            map.insert("apikey".to_string(), Value::String(self.config.api.alphavantage.clone()));
            map.insert("function".to_string(), Value::String(BASE_FUNCTION.to_string()));
        }
        value
    }

    pub async fn poll(&self, args: Arc<Value>) -> Result<Value, ApiError> {
        // Insert API key & the BASE_FUNVTION into the request body.
        let args = self.insert_apikey_and_function(args);
        // Retry the request up to the maximum number of retries.
        let mut retry_count = 0;
        let max_retries = self.config.task.max_retries;
        let delay_ms = self.config.task.base_delay_ms as u64;
        let delay = Duration::from_millis(delay_ms);
        let fetch_type = args.get(FETCH_TYPE_KEY_MAP) // which does not get popped out of the query params
            .and_then(|s| s.as_str())
            .map(FetchType::from_str)
            .unwrap_or(FetchType::Unknown);
        loop {
            match self.get(&fetch_type, BASE_URL, QueryParams::try_from(args.clone())?).await {
                Ok(api_response) => {
                    info!("API GET Response was successfull? : {:?}", bool::from(!api_response.is_null()));
                    return Ok(api_response)
                },
                Err(api_error) => {
                    if retry_count >= max_retries {
                        error!("Failed to fetch data after {} retries.", self.config.task.max_retries);
                        return Err(api_error);
                    }
                    retry_count += 1;
                    // Wait for the retry interval before making the next request
                    tokio::time::sleep(delay).await;
                    warn!("Attempt {}/{} failed with error: {:?}. Retrying in {} seconds.", retry_count, max_retries, api_error, delay_ms);
                    debug!("Retrying request due to error: {}", api_error);
                    // Retry the request
                    continue;
                }
            }
        }
    }
}

/// Example function to demonstrate how to use the Alpha Vantage API.
pub async fn run(client: Arc<Client>, cache: Arc<Mutex<SharedLockedCache>>, config: Arc<ValueConfig>) -> Result<Value, ApiError> {
    // Create configuration.
    // Query parmaters
    let query = QueryParams::new(
        &config.api.alphavantage, 
        BASE_FUNCTION,   // You should not use anything else
        None, // Tickers
        None, // Topics 
        Some(&time_yyyy_mmdd_thhmm(config.request.delay_secs).as_str()), // Time_from 
        None, // Time_to
        None, // Sort
        None  // Limit
    );
    
    // Request Manger
    let req_manager = AlphaVantageApiClient::new(client, cache, config);
    // Make the GET request here.
    let result = req_manager.get_(BASE_URL, query).await
        .map_err(|e| {
            error!("Error during GET request: {}", e); // Log the error
            e // Re-propagate the error without changes
        })?;

    // Return that result
    Ok(result)
}
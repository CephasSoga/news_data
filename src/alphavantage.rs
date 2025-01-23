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
use serde_json::{Value, from_str, to_string};
use reqwest::{Client, Response, StatusCode};
use tracing::{debug, error, info, warn};
use twitter_v2::oauth2::helpers::variant_name;

use crate::config::ValueConfig;
use crate::utils::time_yyyy_mmdd_thhmm;


const BASE_URL: &str = "https://www.alphavantage.co/query";
pub const BASE_FUNCTION: &str = "NEWS_SENTIMENT";

/// Define an abstract error enum.
#[derive(Debug)]
pub enum AbstractApiError {
    /// Abstracts the `BAD_REQUEST` errors.
    RequestError,
    /// Absctracts `Rate Limit Exceeded` errors.
    RateLimitError,
    /// Abstracts `INTERNAL_SERVER_ERROR` errors
    ServerError,
    /// Abstracts `REQUEST_TIMEOUT` errors.
    NetworkError,
    /// Abstracts all other errors,
    UnhandledError,
}

/// Enum for custom error types that extend the `AbstractApiError` Enum.
#[derive(Debug)]
pub enum ApiError {
    /// Represents a request error with optional `status`, `headers` and `body` details.
    RequestError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    /// Represents a rate limit error with optional `status`, `headers` and `body` details.
    RateLimitError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    /// Represents a server error with optional `status`, `headers` and `body` details.
    ServerError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    /// Represents a JSON parsing error.
    JsonParseError {
        message: String,
    },
    /// Represents a network error with optional `status`, `headers` and `body` details.
    NetworkError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    /// Represents an unhandled error with optional `status`, `headers` and `body` details.
    UnhandledError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
}

// Implement Display for ApiError
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RequestError { message, status, headers, body } => {
                write!(f, "Request Error: {} | Status: {:?} | Headers: {:?} | Body: {}", 
                       message, status, headers, body.as_ref().unwrap_or(&"".to_string()))
            }
            ApiError::RateLimitError { message, status, headers, body } => {
                write!(f, "Rate Limit Error: {} | Status: {:?} | Headers: {:?} | Body: {}", 
                       message, status, headers, body.as_ref().unwrap_or(&"".to_string()))
            }
            ApiError::ServerError { message, status, headers, body } => {
                write!(f, "Server Error: {} | Status: {:?} | Headers: {:?} | Body: {}", 
                       message, status, headers, body.as_ref().unwrap_or(&"".to_string()))
            }
            ApiError::JsonParseError { message} => {
                write!(f, "JSON Parse Error: {}", message)
            }
            ApiError::NetworkError { message, status, headers, body } => {
                write!(f, "Network Error: {} | Status: {:?} | Headers: {:?} | Body: {}", 
                       message, status, headers, body.as_ref().unwrap_or(&"".to_string()))
            }
            ApiError::UnhandledError { message, status, headers, body } => {
                write!(f, "Unhandled Error: {} | Status: {:?} | Headers: {:?} | Body: {}", 
                       message, status, headers, body.as_ref().unwrap_or(&"".to_string()))
            }
        }
    }
}

// Implement std::error::Error for ApiError
impl std::error::Error for ApiError {}

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
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        to_string(self)
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


#[derive(Serialize, Deserialize)]
pub struct QueryParams {
    /// The function of your choice. In this case, function=NEWS_SENTIMENT
    pub function: String,

    /// Comma-separated stock/crypto/forex symbols to filter articles (e.g., "IBM").
    /// 
    /// For example: `tickers=IBM` will filter for articles that mention the IBM ticker; 
    /// `tickers=COIN,CRYPTO:BTC,FOREX:USD` will filter for articles that simultaneously mention Coinbase (COIN), 
    /// Bitcoin (CRYPTO:BTC), and US Dollar (FOREX:USD) in their content.
    pub tickers: Option<String>,

    /// Comma-separated topics to filter articles (e.g., "technology").
    ///
    /// ## Available topics:
    ///
    /// - Blockchain: `blockchain`
    /// - Earnings: `earnings`
    /// - IPO: `ipo`
    /// - Mergers & Acquisitions: `mergers_and_acquisitions`
    /// - Financial Markets: `financial_markets`
    /// - Economy - Fiscal Policy (e.g., tax reform, government spending): `economy_fiscal`
    /// - Economy - Monetary Policy (e.g., interest rates, inflation): `economy_monetary`
    /// - Economy - Macro/Overall: `economy_macro`
    /// - Energy & Transportation: `energy_transportation`
    /// - Finance: `finance`
    /// - Life Sciences: `life_sciences`
    /// - Manufacturing: `manufacturing`
    /// - Real Estate & Construction: `real_estate`
    /// - Retail & Wholesale: `retail_wholesale`
    /// - Technology: `technology`
    pub topics: Option<String>,

    /// Start time for filtering articles in YYYYMMDDTHHMM format.
    /// 
    /// For example: time_from=20220410T0130.
    pub time_from: Option<String>,

    /// End time for filtering articles in YYYYMMDDTHHMM format.
    /// 
    /// If time_from is specified but time_to is missing, 
    /// the API will return articles published between the time_from value and the current time
    pub time_to: Option<String>,

    /// Sort order: "LATEST", "EARLIEST", or "RELEVANCE".
    pub sort: Option<String>,

    /// Maximum number of results to return (default is 50).
    /// You can also set limit=1000 to output up to 1000 results.           
    pub limit: Option<i32>,

    /// Your Alpha Vantage API key. Claim your free API Key [here](https://www.alphavantage.co/support/#api-key).             
    pub apikey: String,                    
}

impl QueryParams {
    pub fn new(
        apikey: &str,
        function: &str,
        tickers: Option<&str>,
        topics: Option<&str>,
        time_from: Option<&str>,
        time_to: Option<&str>,
        sort: Option<&str>,
        limit: Option<i32>,
    ) -> Self {
        Self {
            function: function.to_string(),
            tickers: tickers.map(|t| t.to_string()),
            topics: topics.map(|t| t.to_string()),
            time_from: time_from.map(|t| t.to_string()),
            time_to: time_to.map(|t| t.to_string()),
            sort: sort.map(|s| s.to_string()),
            limit: limit,
            apikey: apikey.to_string(),                   
        }                                                       
    }
}
impl TryFrom<Value> for QueryParams {
    type Error = ApiError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|err| ApiError::JsonParseError { message: err.to_string() })
    }    
}

pub struct AlphaVantageApiClient {
    client: Arc<Client>,
    config: Arc<ValueConfig>,
}
impl AlphaVantageApiClient {
        pub fn new(client: Arc<Client>, config: Arc<ValueConfig>) -> Self {
        Self {client, config}
    }
    pub async fn get(
        &self, 
        url: &str, 
        query_params: QueryParams
    ) -> Result<AlphaVantageApiResponse, ApiError> {
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

        // Attempt to parse the JSON response directly
        let response_json: AlphaVantageApiResponse = response.json().await.map_err(|e| {
            error!("Failed to read body: {:?}", e);
            ApiError::JsonParseError { message: e.to_string() }
        })?; // Handle JSON parsing error

        Ok(response_json)
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

    fn insert_apikey_and_function(&self, mut value: Value) -> Value {
        if let Value::Object(ref mut map) = value {
            map.insert("apikey".to_string(), Value::String(self.config.api.alphavantage.clone()));
            map.insert("function".to_string(), Value::String(BASE_FUNCTION.to_string()));
        }
        value
    }

    pub async fn poll(&self, args: Value) -> Result<AlphaVantageApiResponse, ApiError> {
        // Insert API key & the BASE_FUNVTION into the request body.
        let args = self.insert_apikey_and_function(args);
        // Retry the request up to the maximum number of retries.
        let mut retry_count = 0;
        let max_retries = self.config.task.max_retries;
        let delay_ms = self.config.task.base_delay_ms as u64;
        let delay = Duration::from_millis(delay_ms);
        loop {
            match self.get(BASE_URL, QueryParams::try_from(args.clone())?).await {
                Ok(api_response) => {
                    info!("API GET Response was successfull? : {:?}", bool::from(!api_response.feed.is_empty()));
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
pub async fn run(client: Arc<Client>, value_config: Arc<ValueConfig>) -> Result<AlphaVantageApiResponse, ApiError> {
    // Create configuration.
    // Query parmaters
    let query = QueryParams::new(
        &value_config.api.alphavantage, 
        BASE_FUNCTION,   // You should not use anything else
        None, // Tickers
        None, // Topics 
        Some(&time_yyyy_mmdd_thhmm(value_config.request.delay_secs).as_str()), // Time_from 
        None, // Time_to
        None, // Sort
        None  // Limit
    );
    
    // Request Manger
    let req_manager = AlphaVantageApiClient::new(client, value_config);
    // Make the GET request here.
    let result = req_manager.get(BASE_URL, query).await
        .map_err(|e| {
            error!("Error during GET request: {}", e); // Log the error
            e // Re-propagate the error without changes
        })?;

    // Return that result
    Ok(result)
}
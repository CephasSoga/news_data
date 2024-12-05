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

// API Parameters

// ❚ Required: function
// The function of your choice. In this case, function=NEWS_SENTIMENT

// ❚ Optional: tickers
// The stock/crypto/forex symbols of your choice. 
// For example: tickers=IBM will filter for articles that mention the IBM ticker;
// tickers=COIN,CRYPTO:BTC,FOREX:USD will filter for articles that simultaneously mention 
// Coinbase (COIN), Bitcoin (CRYPTO:BTC), and US Dollar (FOREX:USD) in their content.

// ❚ Optional: topics
// The news topics of your choice. For example: topics=technology will filter for articles 
// that write about the technology sector; topics=technology,ipo will filter for articles 
// that simultaneously cover technology and IPO in their content.
// Below is the full list of supported topics:
// Blockchain: blockchain
// Earnings: earnings
// IPO: ipo
// Mergers & Acquisitions: mergers_and_acquisitions
// Financial Markets: financial_markets
// Economy - Fiscal Policy (e.g., tax reform, government spending): economy_fiscal
// Economy - Monetary Policy (e.g., interest rates, inflation): economy_monetary
// Economy - Macro/Overall: economy_macro
// Energy & Transportation: energy_transportation
// Finance: finance
// Life Sciences: life_sciences
// Manufacturing: manufacturing
// Real Estate & Construction: real_estate
// Retail & Wholesale: retail_wholesale
// Technology: technology

// ❚ Optional: time_from and time_to
// The time range of the news articles you are targeting, in YYYYMMDDTHHMM format. 
// For example: time_from=20220410T0130. If time_from is specified but time_to is missing, 
// the API will return articles published between the time_from value and the current time.

// ❚ Optional: sort
// By default, sort=LATEST and the API will return the latest articles first. 
// You can also set sort=EARLIEST or sort=RELEVANCE based on your use case.

// ❚ Optional: limit
// By default, limit=50 and the API will return up to 50 matching results. 
// You can also set limit=1000 to output up to 1000 results.

// ❚ Required: apikey
// Your API key.

#[allow(dead_code)]
#[allow(unused_imports)]

use std::env;
use std::fmt;
use std::collections::HashMap;

use dotenv;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use reqwest::{Client, Response, StatusCode};



/// Define an abstract error enum
#[derive(Debug)]
pub enum AbstractApiError {
    RequestError,
    RateLimitError,
    ServerError,
    NetworkError,
    UnhandledError,
}

/// Define custom error types which exttends the `AbstractApiError` Enum
#[derive(Debug)]
pub enum ApiError {
    RequestError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    RateLimitError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    ServerError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
    JsonParseError {
        message: String,
    },
    NetworkError {
        message: String,
        status: Option<StatusCode>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    },
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

#[derive(Debug, Serialize, Deserialize)]

/// Wrapper of the Alpha Vantage API response.
/// 
/// [See example here](https://www.alphavantage.co/query?function=NEWS_SENTIMENT&tickers=AAPL&apikey=demo).
pub struct AlphaVantageApiResponse {
    pub items: String,
    pub sentiment_score_definition: String,
    pub relevance_score_definition: String,
    pub feed: Vec<FeedItem>,
}
impl AlphaVantageApiResponse {
    /// Converts the response into a JSON-compatible HashMap
    pub fn to_json(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();

        // Insert basic fields as key-value pairs
        map.insert("items".to_string(), Value::String(self.items.clone()));
        map.insert(
            "sentiment_score_definition".to_string(),
            Value::String(self.sentiment_score_definition.clone()),
        );
        map.insert(
            "relevance_score_definition".to_string(),
            Value::String(self.relevance_score_definition.clone()),
        );

        // Serialize the feed into a JSON array
        map.insert(
            "feed".to_string(),
            Value::Array(self.feed.iter().map(|item| serde_json::to_value(item).unwrap()).collect()),
        );

        map
    }

    /// Converts a JSON string to an `AlphaVantageApiResponse` instance.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<AlphaVantageApiResponse>(json_str)
    }

    /// Converts a `HashMap<String, Value>` to an `AlphaVantageApiResponse` instance.
    pub fn from_hashmap(map: HashMap<String, Value>) -> Result<Self, serde_json::Error> {
        // Convert HashMap into a JSON string and deserialize it into the struct
        let json_value = Value::Object(map.into_iter().collect());
        serde_json::from_value(json_value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedItem {
    pub title: String,
    pub url: String,
    pub time_published: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub banner_image: String,
    pub source: String,
    pub category_within_source: String,
    pub source_domain: String,
    pub topics: Vec<Topic>,
    pub overall_sentiment_score: f64,
    pub overall_sentiment_label: String,
    pub ticker_sentiment: Vec<TickerSentiment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    pub topic: String,
    pub relevance_score: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerSentiment {
    pub ticker: String,
    pub relevance_score: String,
    pub ticker_sentiment_score: String,
    pub ticker_sentiment_label: String,
}

/// Refers to the requests configuration. This sruct centralizes theconfiguration control.
/// 
/// ## Why This Works:
///
/// - Centralized API Key Management:
///
/// RequestConfig handles the API key, ensuring it is defined and retrieved consistently across the application.
/// 
/// - Ease of Use:
///
/// Callers only need to provide the RequestConfig instance, reducing the risk of forgetting to include the API key manually.
/// Improved Maintainability:
///
/// If the way the API key is retrieved changes (e.g., from an environment variable to a configuration file), only the RequestConfig implementation needs updating.
/// 
pub struct  RequestConfig{
    apikey: String,
    base_url: String,
}
impl RequestConfig {
    /// Creates a new instance of `RequestConfig`.
    /// 
    /// Reads API key form env variables. So Make sure you set **ALPHA_VANTAGE_API_KEY** .
    pub fn new() -> Self {
        // Load environment variables from `.env` file
        dotenv::dotenv().ok();

        // Retrieve the API key from the environment
        let apikey = std::env::var("ALPHA_VANTAGE_API_KEY")
            .expect("ALPHA_VANTAGE_API_KEY env variable is not set!");

        // Define the base URL for API requests
        let base_url = String::from("https://www.alphavantage.co");

        Self { apikey, base_url }
    }
}

#[derive(Serialize, Deserialize)]
/// Refers to the HTTP request parameters for Alpha Vantage
pub struct RequestParams {
    pub path_params: PathParams,
    pub query_params: QueryParams,
}

#[derive(Serialize, Deserialize)]
/// Represents the path parameters in an API request
pub struct PathParams {
    pub endpoint: String, // API endpoint, e.g., "query"
}

impl PathParams {
    pub fn new(req_config: RequestConfig) -> Self {
        Self {
            endpoint: req_config.base_url,
        }
    }
}

#[derive(Serialize, Deserialize)]
/// Represents the query parameters in an API request
pub struct QueryParams {
    pub function: String,                  // In this case, function=NEWS_SENTIMENT
    pub tickers: Option<String>,           // For example: tickers=IBM will filter for articles that mention the IBM ticker
    pub topics: Option<String>,            // For example: topics=technology will filter for articles that write about the technology sector
    pub time_from: Option<String>,         // The start of the time range of the news articles you are targeting, in YYYYMMDDTHHMM format
    pub time_to: Option<String>,           // The end of the time range of the news articles you are targeting, in YYYYMMDDTHHMM format
    pub sort: Option<String>,              // Can be set to LATEST, EARLIEST or RELEVANCE based on your use case
    pub limit: Option<i32>,                // The count you want for outputs. Defaults to 50
    pub apikey: String,                    // Your Alpha vantage API key
}

impl QueryParams {
    /// Creates a new instance of QueryParams with required and optional parameters.
    pub fn new(
        req_config: &RequestConfig,
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
            apikey: req_config.apikey.clone()                   
        }                                                       
    }

    /// Sets the tickers filter
    pub fn set_tickers(&mut self, tickers: &str) {
        self.tickers = Some(tickers.to_string());
    }

    /// Sets the topics filter
    pub fn set_topics(&mut self, topics: &str) {
        self.topics = Some(topics.to_string());
    }

    /// Sets the time_from parameter
    pub fn set_time_from(&mut self, time_from: &str) {
        self.time_from = Some(time_from.to_string());
    }

    /// Sets the time_to parameter
    pub fn set_time_to(&mut self, time_to: &str) {
        self.time_to = Some(time_to.to_string());
    }

    /// Sets the sort order
    pub fn set_sort(&mut self, sort: &str) {
        self.sort = Some(sort.to_string());
    }

    /// Sets the limit for the number of results
    pub fn set_limit(&mut self, limit: i32) {
        self.limit = Some(limit);
    }
}

pub struct RequestManager {
    client: Client
}
impl RequestManager {
    pub fn new(client: Client) -> Self {
        Self {client}
    }

    /// Sends a GET request to the Alpha Vantage API with the provided path and query parameters.
    /// 
    /// This function constructs the full URL by combining the base URL with the provided path parameters.
    /// It then sends a GET request to this URL with the provided query parameters. The response is
    /// parsed into an `AlphaVantageApiResponse` instance, which is then returned.
    /// 
    /// ## Arguments:
    /// 
    /// - `path_params`: The path parameters for the API request.
    /// - `query_params`: The query parameters for the API request.
    /// 
    /// ## Returns:
    /// 
    /// The response from the Alpha Vantage API, parsed into an `AlphaVantageApiResponse` instance.
    /// 
    /// ## Errors:
    /// 
    /// This function can return the following errors:
    /// 
    /// - `ApiError::NetworkError`: If there is a network error while sending the request.
    /// - `ApiError::RequestError`: If there is a general request error.
    /// - `ApiError::RateLimitError`: If the rate limit has been exceeded.
    /// - `ApiError::ServerError`: If there is a server error.
    /// - `ApiError::JsonParseError`: If there is an error parsing the JSON response.
    /// 
    /// # Example
    /// 
    /// ```
    /// let config = RequestConfig::new();
    /// let request_manager = RequestManager::new(config);
    /// let path_params = PathParams::new(config);
    /// let query_params = QueryParams::new(config);
    /// let response = request_manager.get(path_params, query_params).await;
    /// ```
    ///
    pub async fn get(
        &self, 
        path_params: PathParams, 
        query_params: QueryParams
    ) -> Result<AlphaVantageApiResponse, ApiError> {
        
        // Construct URL
        let url = path_params.endpoint;

        // Send GET request
        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await.map_err(|e| {
                // Check if the error is a network error
                if e.is_timeout() || e.is_connect() {
                    ApiError::NetworkError {
                        message: e.to_string(),
                        status: Some(StatusCode::GATEWAY_TIMEOUT),
                        headers: None,
                        body: None,
                    }
                } else {
                    ApiError::RequestError{
                        message: e.to_string(),
                        status: Some(StatusCode::BAD_REQUEST),
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

        let response_json = response.json::<AlphaVantageApiResponse>()
            .await.map_err(|e| ApiError::JsonParseError{message: e.to_string(),})?; // Handle JSON parsing error

        Ok(response_json)
    }

    /// Parses the response error from the Alpha Vantage API and constructs an appropriate `ApiError`.
    /// 
    /// This function is called when an error occurs during an API request. It extracts the status, headers,
    /// and body from the response and maps them to a specific `ApiError` variant based on the provided
    /// `abstract_error_type`.
    /// 
    /// ## Arguments:
    /// 
    /// - `message`: A string containing the error message to be included in the `ApiError`.
    /// - `response`: The `Response` object from the `reqwest` library, which contains details about the HTTP response.
    /// - `abstract_error_type`: An enum variant of `AbstractApiError` that indicates the type of error encountered.
    /// 
    /// ## Returns:
    /// 
    /// This function returns an `ApiError` instance that corresponds to the type of error encountered,
    /// populated with the relevant details from the response.
    /// 
    /// ## Panics:
    /// 
    /// If an unsupported error type is provided, the function will panic with a message indicating that
    /// the error type is not supported.
    /// 
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
}
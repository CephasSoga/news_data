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
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_string};
use reqwest::{Client, Response, StatusCode};

use crate::config::ValueConfig;
use crate::utils::time_yyyy_mmdd_thhmm;



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
    ///
    /// # Arguments
    ///
    /// * `json` - A string slice that holds the JSON data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the `AlphaVantageApiResponse` or a `serde_json::Error`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        from_str(json)
    }

    /// Serializes the `AlphaVantageApiResponse` to a JSON string.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the JSON string or a `serde_json::Error`.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        to_string(self)
    }

    /// Constructs a `AlphaVantageApiResponse` from a HashMap.
    ///
    /// # Arguments
    ///
    /// * `map` - A HashMap containing the data to be converted.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the `AlphaVantageApiResponse` or a `serde_json::Error`.
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
    /// Reads API key form Config.toml, somake sure you have that set and matching the format inside `./config.toml` .
    pub fn new(value_config: &ValueConfig) -> Self {
        let apikey = value_config.api.alphavantage.clone();

        // Define the base URL for API requests
        let base_url = String::from("https://www.alphavantage.co/query");

        Self { apikey, base_url }
    }
}

#[derive(Serialize, Deserialize)]
/// Refers to the HTTP request parameters for Alpha Vantage.
pub struct RequestParams {
    pub path_params: PathParams,
    pub query_params: QueryParams,
}

#[derive(Serialize, Deserialize)]
/// Represents the path parameters in an API request.
///
/// This struct is used to encapsulate the endpoint URL for API requests,
/// allowing for easier management and modification of the endpoint as needed.
pub struct PathParams {
    pub endpoint: String,
}
impl PathParams {
    /// Creates a new instance of `PathParams`.
    ///
    /// # Arguments
    ///
    /// * `req_config` - A `RequestConfig` instance that contains the base URL for the API.
    ///
    /// # Returns
    ///
    /// Returns a `PathParams` instance initialized with the endpoint derived from the provided `RequestConfig`.
    pub fn new(req_config: &RequestConfig) -> Self {
        Self {
            endpoint: req_config.base_url.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
/// Represents the query parameters in an API request
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
    /// Creates a new instance of `QueryParams` with required and optional parameters.
    ///
    /// # Parameters
    /// 
    /// - `req_config`: A reference to the `RequestConfig` instance containing the API key.
    /// - `function`: The function to be called (e.g., "NEWS_SENTIMENT").
    /// - `tickers`: Optional comma-separated stock/crypto/forex symbols to filter articles.
    /// - `topics`: Optional comma-separated topics to filter articles.
    /// - `time_from`: Optional start time for filtering articles in `YYYYMMDDTHHMM` format.
    /// - `time_to`: Optional end time for filtering articles in `YYYYMMDDTHHMM` format.
    /// - `sort`: Optional sort order ("LATEST", "EARLIEST", or "RELEVANCE").
    /// - `limit`: Optional maximum number of results to return (default is 50).
    ///
    /// # Returns
    ///
    /// Returns a `QueryParams` instance populated with the provided parameters.
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
    
    /// Creates a new instance of `RequestManager`.
    /// 
    /// # Arguments
    /// 
    /// * `client` - An instance of `reqwest::Client` used to send HTTP requests.
    /// 
    /// # Returns
    /// 
    /// Returns a `RequestManager` instance initialized with the provided client.
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
            eprintln!("Failed to read body: {:?}", e);
            ApiError::JsonParseError { message: e.to_string() }
        })?; // Handle JSON parsing error

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

/// Example function to demonstrate how to use the Alpha Vantage API.
///
/// This function initializes the request configuration, constructs the path and query parameters,
/// creates a new HTTP client, and sends a GET request to the Alpha Vantage API for news sentiment data.
///
/// ## Argument:
/// 
/// - value_config (&ValueConfig): this holds the variables extracted from the config.toml file.
/// We use it for flexibility and we always pass it as an argument so that we can avoid reading
/// the configuration file every single time functions are called.
/// ## Returns
///
/// Returns a `Result` containing either:
/// - `AlphaVantageApiResponse`: The response from the API containing news sentiment data.
/// - `ApiError`: An error if the request fails or if there is an issue with the response.
///
/// ## Errors
///
/// This function can return various errors, including:
/// - `ApiError::NetworkError`: If there is a network issue while sending the request.
/// - `ApiError::RequestError`: If there is a general request error.
/// - `ApiError::RateLimitError`: If the rate limit for API requests has been exceeded.
/// - `ApiError::ServerError`: If there is a server error from the API.
/// - `ApiError::JsonParseError`: If there is an error parsing the JSON response.
///
/// ## Example
///
/// ```
/// let result = example().await;
/// match result {
///     Ok(response) => println!("Received response: {:?}", response),
///     Err(e) => eprintln!("Error occurred: {}", e),
/// }
/// ```
pub async fn run(value_config: &ValueConfig) -> Result<AlphaVantageApiResponse, ApiError> {
    // Create configuration.
    let config = RequestConfig::new(value_config);
    // Path parameters
    let path = PathParams::new(&config);
    // Query parmaters
    let query = QueryParams::new(
        &config, 
        "NEWS_SENTIMENT",   // You should not use anything else
        None, // Tickers
        None, // Topics 
        Some(&time_yyyy_mmdd_thhmm(value_config.request.delay_secs).as_str()), // Time_from 
        None, // Time_to
        None, // Sort
        None  // Limit
    );
    
    // Request Client
    let client = Client::new();
    // Request Manger
    let req_manager = RequestManager::new(client);
    // Make the GET request here.
    let result = req_manager.get(path, query).await
        .map_err(|e| {
            eprintln!("Error during GET request: {}", e); // Log the error
            e // Re-propagate the error without changes
        })?;

    println!("Result of GET request: {:?}", result.feed.len());

    // Return that result
    Ok(result)
}
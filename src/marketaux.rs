//! ## A Rust wrapper of the [Marketaux API](https://www.marketaux.com).
//! 
//! Get all the latest global financial news and filter by entities identified within articles to build concise news feeds. 
//! Also provided is analysis of each entity identified in articles. Note that not every article may have entities identified. 
//! To retrieve all news for articles with identified entities, use the parameter must_have_entities, 
//! or specify any of the entity params such as symbols or exchanges as defined below to produce more concise results.
//! 
//! ## Reference:
//! [Official Marketaux Documentation](https://www.marketaux.com/documentation).
//! 

use std::fmt;
use std::hash::{Hash, Hasher};

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_string};

use crate::config::ValueConfig;
use crate::utils::time_rfc3339_opts;

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

// Implement std::error::Error for ApiError.
impl std::error::Error for ApiError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Represents the response from the Marketaux API.
///
/// This struct contains metadata about the response and the actual data (news items).
/// 
/// [See example here](https://www.marketaux.com/documentation).
pub struct MarketAuxResponse {
    pub meta: Meta,
    pub data: Vec<NewsItem>,
}

impl Hash for MarketAuxResponse {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}
impl PartialEq for MarketAuxResponse {
    fn eq(&self, other: &Self) -> bool {
        self.meta == other.meta && self.data == other.data // Ensure both fields are comparable
    }
}
impl MarketAuxResponse {
    /// Constructs a `MarketAuxResponse` from a JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - A string slice that holds the JSON data.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the `MarketAuxResponse` or a `serde_json::Error`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        from_str(json)
    }

    /// Serializes the `MarketAuxResponse` to a JSON string.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the JSON string or a `serde_json::Error`.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        to_string(self)
    }

    /// Constructs a `MarketAuxResponse` from a HashMap.
    ///
    /// # Arguments
    ///
    /// * `map` - A HashMap containing the data to be converted.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the `MarketAuxResponse` or a `serde_json::Error`.
    pub fn from_hashmap(map: std::collections::HashMap<String, serde_json::Value>) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(&map)?;
        Self::from_json(&json)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
    pub found: i64,
    pub returned: i64,
    pub limit: i64,
    pub page: i64,
}

impl PartialEq for Meta {
    fn eq(&self, other: &Self) -> bool {
        self.found == other.found &&
        self.returned == other.returned &&
        self.limit == other.limit &&
        self.page == other.page
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsItem {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub snippet: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub language: Option<String>,
    #[serde(rename = "published_at")]
    pub published_at: Option<String>, // you can change this to DateTime if needed
    pub source: Option<String>,
    pub relevance_score: Option<f64>,
    pub entities: Vec<Entity>,
    pub similar: Vec<Value>, // Assuming similar items can vary in structure
}

impl Hash for NewsItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl PartialEq for NewsItem {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid &&
        self.title == other.title
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub exchange_long: Option<String>,
    pub country: Option<String>,
    pub r#type: Option<String>, // Using `r#type` to avoid conflicting with the `type` keyword
    pub industry: Option<String>,
    pub match_score: f64,
    pub sentiment_score: f64,
    pub highlights: Vec<Highlight>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub highlight: Option<String>,
    pub sentiment: Option<f64>,
    #[serde(rename = "highlighted_in")]
    pub highlighted_in: Option<String>,
}


pub struct RequestConfig {
    apikey: String,
    base_url: String
}
impl RequestConfig {
    /// Creates a new instance of `RequestConfig`.
    /// 
    /// This method reads the Marketaux API key from your config.toml.
    /// You want to make sure that file is all set and matching the stucture inside the config.rs file. 
    ///
    /// The method also defines the base URL for API requests, which is set to "https://api.marketaux.com/v1/news".
    ///
    /// ## Returns:
    ///
    /// Returns an instance of `RequestConfig` containing the API key and base URL.
    pub fn new(value_config: &ValueConfig) -> Self {
        
        // Reads API key from values configuration file
        let apikey = value_config.api.marketaux.clone();

        // Define the base URL for API requests
        let base_url = String::from("https://api.marketaux.com/v1/news");

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
/// Represents the path parameters in an API request.
/// 
/// ## Attribute:
/// 
/// - url: the String of the full url to use for the GET request.
/// 
pub struct PathParams {
    /// Formats Base Url and API Endpoint in a single string object.
    pub url: String,
}
impl PathParams {
    
    /// Creates a new instance of `PathParams`.
    ///
    /// This method constructs a `PathParams` object by combining the base URL from the provided
    /// `RequestConfig` and the specified endpoint. It formats the URL to create a complete path
    /// for API requests.
    ///
    /// ## Arguments:
    ///
    /// - `req_config`: An instance of `RequestConfig` containing the base URL for the API.
    /// - `endpoint`: A `String` representing the specific endpoint to be appended to the base URL.
    /// 
    /// `Options`: **all**, **similar**, **uuid**.
    /// 
    /// `**Notes**`: When you pass `uuid` as endpoint argument, you should not use any other query paramaters than your api token in the GET request.
    ///
    /// ## Returns:
    ///
    /// Returns a new instance of `PathParams` with the constructed URL.
    pub fn new(req_config: &RequestConfig, endpoint: String) -> Self {
        Self {
            url: format!("{}/{}", req_config.base_url, endpoint),
        }
    }
}


#[derive(Serialize, Deserialize)]
/// Represents the HTTP request parameters for the Marketaux API.
///
/// This struct contains all the parameters that can be used to customize the API request
/// to fetch financial news articles. Each field corresponds to a specific query parameter
/// that can be included in the request.
pub struct QueryParams {
    /// Your Marketaux API key.
    api_token: String,

    /// Specify entity symbol(s) identified within the article.
    /// Example: symbols=TSLA,AMZN,MSFT
    symbols: Option<String>,

    /// Specify the type of entities identified within the article.
    /// Example: entity_types=index,equity
    entity_types: Option<String>,

    /// Specify the industries of entities identified within the article.
    /// Example: industries=Technology,Industrials
    industries: Option<String>,

    /// Specify the country of the exchange for identified entities within the article.
    /// Example: countries=us,ca
    countries: Option<String>,

    /// Find articles with entities having a sentiment score greater than or equal to x.
    /// Example: sentiment_gte=0 - Finds articles that are neutral or positive.
    sentiment_gte: Option<i32>,

    /// Find articles with entities having a sentiment score less than or equal to x.
    /// Example: sentiment_lte=0 - Finds articles that are neutral or negative.
    sentiment_lte: Option<i32>,

    /// Find articles with entities having a match score greater than or equal to min_match_score.
    min_match_score: Option<f32>,

    /// By default, all entities for each article are returned.
    /// Set this to true to return only relevant entities for your query.
    /// Example: filter_entities=true (Only relevant entities will be returned).
    filter_entities: Option<bool>,

    /// Set to true to ensure at least one entity is identified within the article.
    /// By default, all articles are returned. Defaults to FALSE.
    must_have_entities: Option<bool>,

    /// Group similar articles to avoid displaying multiple articles on the same topic/subject.
    /// Default is true.
    group_similar: Option<bool>,

    /// Use to search for specific terms or phrases in articles.
    /// Supports advanced query usage with operators (+, |, -, ", *, ( ) )
    /// Example: search="ipo" -nyse (Searches for articles mentioning "ipo" but not NYSE).
    search: Option<String>,

    /// Specify a comma-separated list of domains to include in the search.
    /// Example: domains=adweek.com,adage.com
    domains: Option<String>,

    /// Specify a comma-separated list of domains to exclude from the search.
    /// Example: exclude_domains=example.com
    exclude_domains: Option<String>,

    /// Specify a comma-separated list of source IDs to include in the search.
    /// Example: source_ids=adweek.com-1,adage.com-1
    source_ids: Option<String>,

    /// Specify a comma-separated list of source IDs to exclude from the search.
    exclude_source_ids: Option<String>,

    /// Specify a comma-separated list of languages to include. Default is all languages.
    /// Example: language=en,es (Includes English and Spanish articles).
    language: Option<String>,

    /// Find articles published before the specified date.
    /// Example: published_before=2024-12-05T08:25:06
    published_before: Option<String>,

    /// Find articles published after the specified date.
    /// Example: published_after=2024-12-05T08:25:06
    published_after: Option<String>,

    /// Find articles published on the specified date.
    /// Example: published_on=2024-12-05
    published_on: Option<String>,

    /// Sort articles by published date, entity match score, entity sentiment score, or relevance score.
    /// Example: sort=entity_match_score
    sort: Option<String>,

    /// Specify the sort order for the sort parameter. Options: "desc" | "asc".
    /// Default is "desc".
    sort_order: Option<String>,

    /// Specify the number of articles to return. Default is the maximum specified for your plan.
    /// Example: limit=50
    limit: Option<i32>,

    /// Use for pagination to navigate through the result set. Default is 1.
    /// Example: page=2
    page: Option<i32>,
}

impl QueryParams {
    /// Creates a new instance of QueryParams with required and optional parameters.
    pub fn new(
        req_config: &RequestConfig,
        symbols: Option<&str>,
        entity_types: Option<&str>,
        industries: Option<&str>,
        countries: Option<&str>,
        sentiment_gte: Option<i32>,
        sentiment_lte: Option<i32>,
        min_match_score: Option<f32>,
        filter_entities: Option<bool>,
        must_have_entities: Option<bool>,
        group_similar: Option<bool>,
        search: Option<&str>,
        domains: Option<&str>,
        exclude_domains: Option<&str>,
        source_ids: Option<&str>,
        exclude_source_ids: Option<&str>,
        language: Option<&str>,
        published_before: Option<&str>,
        published_after: Option<&str>,
        published_on: Option<&str>,
        sort: Option<&str>,
        sort_order: Option<&str>,
        limit: Option<i32>,
        page: Option<i32>,
    ) -> Self {
        Self {
            api_token: req_config.apikey.clone(),
            symbols: symbols.map(|s| s.to_string()),
            entity_types: entity_types.map(|s| s.to_string()),
            industries: industries.map(|s| s.to_string()),
            countries: countries.map(|s| s.to_string()),
            sentiment_gte,
            sentiment_lte,
            min_match_score,
            filter_entities,
            must_have_entities,
            group_similar,
            search: search.map(|s| s.to_string()),
            domains: domains.map(|s| s.to_string()),
            exclude_domains: exclude_domains.map(|s| s.to_string()),
            source_ids: source_ids.map(|s| s.to_string()),
            exclude_source_ids: exclude_source_ids.map(|s| s.to_string()),
            language: language.map(|s| s.to_string()),
            published_before: published_before.map(|s| s.to_string()),
            published_after: published_after.map(|s| s.to_string()),
            published_on: published_on.map(|s| s.to_string()),
            sort: sort.map(|s| s.to_string()),
            sort_order: sort_order.map(|s| s.to_string()),
            limit,
            page,
        }
    }

    /// Sets the symbols filter.
    pub fn set_symbols(&mut self, symbols: &str) {
        self.symbols = Some(symbols.to_string());
    }

     /// Sets the entity_types filter.
     pub fn set_entity_types(&mut self, entity_types: &str) {
        self.entity_types = Some(entity_types.to_string());
    }

    /// Sets the industries filter.
    pub fn set_industries(&mut self, industries: &str) {
        self.industries = Some(industries.to_string());
    }

    /// Sets the countries filter.
    pub fn set_countries(&mut self, countries: &str) {
        self.countries = Some(countries.to_string());
    }

    /// Sets the sentiment_gte filter.
    pub fn set_sentiment_gte(&mut self, sentiment: i32) {
        self.sentiment_gte = Some(sentiment);
    }

    /// Sets the sentiment_lte filter.
    pub fn set_sentiment_lte(&mut self, sentiment: i32) {
        self.sentiment_lte = Some(sentiment);
    }

    /// Sets the min_match_score filter.
    pub fn set_min_match_score(&mut self, score: f32) {
        self.min_match_score = Some(score);
    }

    /// Sets the filter_entities flag.
    pub fn set_filter_entities(&mut self, filter: bool) {
        self.filter_entities = Some(filter);
    }

    /// Sets the must_have_entities flag.
    pub fn set_must_have_entities(&mut self, must_have: bool) {
        self.must_have_entities = Some(must_have);
    }

    /// Sets the group_similar flag.
    pub fn set_group_similar(&mut self, group: bool) {
        self.group_similar = Some(group);
    }

    /// Sets the search filter.
    pub fn set_search(&mut self, search: &str) {
        self.search = Some(search.to_string());
    }

    /// Sets the domains filter.
    pub fn set_domains(&mut self, domains: &str) {
        self.domains = Some(domains.to_string());
    }

    /// Sets the exclude_domains filter.
    pub fn set_exclude_domains(&mut self, exclude: &str) {
        self.exclude_domains = Some(exclude.to_string());
    }

    /// Sets the source_ids filter.
    pub fn set_source_ids(&mut self, source_ids: &str) {
        self.source_ids = Some(source_ids.to_string());
    }

    /// Sets the exclude_source_ids filter.
    pub fn set_exclude_source_ids(&mut self, exclude: &str) {
        self.exclude_source_ids = Some(exclude.to_string());
    }

    /// Sets the language filter.
    pub fn set_language(&mut self, language: &str) {
        self.language = Some(language.to_string());
    }

    /// Sets the published_before filter.
    pub fn set_published_before(&mut self, date: &str) {
        self.published_before = Some(date.to_string());
    }

    /// Sets the published_after filter.
    pub fn set_published_after(&mut self, date: &str) {
        self.published_after = Some(date.to_string());
    }

    /// Sets the published_on filter.
    pub fn set_published_on(&mut self, date: &str) {
        self.published_on = Some(date.to_string());
    }

    /// Sets the sort filter.
    pub fn set_sort(&mut self, sort: &str) {
        self.sort = Some(sort.to_string());
    }

    /// Sets the sort_order filter.
    pub fn set_sort_order(&mut self, order: &str) {
        self.sort_order = Some(order.to_string());
    }

    /// Sets the limit filter.
    pub fn set_limit(&mut self, limit: i32) {
        self.limit = Some(limit);
    }

    /// Sets the page filter.
    pub fn set_page(&mut self, page: i32) {
        self.page = Some(page);
    }
}

pub struct RequestManager {
    client: Client
}
impl RequestManager {

    pub fn new(client: Client) -> Self {
        Self {client}
    }

    /// Sends a GET request to the specified API endpoint with the provided path and query parameters.
    ///
    /// This method constructs a GET request using the `reqwest` client and handles the response.
    /// It checks for various error conditions, including network errors, rate limit errors,
    /// server errors, and unhandled errors. If the request is successful, it attempts to parse
    /// the response into a `MarketAuxResponse` object.
    ///
    /// ## Arguments:
    ///
    /// - `path_params`: The path parameters containing the URL for the API endpoint.
    /// - `query_params`: The query parameters to be included in the request.
    ///
    /// ## Returns:
    ///
    /// Returns a `Result` containing either a `MarketAuxResponse` on success or an `ApiError`
    /// on failure.
    ///
    /// ## Errors:
    ///
    /// This method may return various `ApiError` variants, including:
    /// - `NetworkError`: If there is a network-related issue.
    /// - `RateLimitError`: If the rate limit for API requests has been exceeded.
    /// - `ServerError`: If the server returns a 5xx status code.
    /// - `UnhandledError`: For any other non-200 status codes.
    /// - `JsonParseError`: If the response cannot be parsed into the expected format.
    pub async fn get(
        &self,
        path_params: PathParams,
        query_params: Option<QueryParams>
    ) -> Result<MarketAuxResponse, ApiError> {
            // Send GET request
            let response = self
            .client
            .get(&path_params.url)
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
        let response_json: MarketAuxResponse = response.json().await.map_err(|e| {
            eprintln!("Failed to read body: {:?}", e);
            ApiError::JsonParseError { message: e.to_string() }
        })?; // Handle JSON parsing error

        Ok(response_json)
    }

    /// Parses the response error from the Marketaux API and constructs an appropriate `ApiError`.
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


/// Asynchronously fetches financial news articles from the Marketaux API.
///
/// This function serves as an example of how to use the `RequestConfig`, `PathParams`, and `QueryParams`
/// structs to construct a request to the Marketaux API. It initializes the necessary configurations,
/// constructs the request parameters, and sends a GET request to retrieve news articles.
///
/// ## Argument:
/// 
/// - value_config (&ValueConfig): this holds the variables extracted from the config.toml file.
/// We use it for flexibility and we always pass it as an argument so that we can avoid reading
/// the configuration file every single time functions are called.
/// 
/// ## Returns
///
/// Returns a `Result` containing either:
/// - `MarketAuxResponse`: A successful response containing the fetched news articles and metadata.
/// - `ApiError`: An error that occurred during the request process, which can include network errors,
///   rate limit errors, server errors, or JSON parsing errors.
///
/// ## Example
///
/// ```rust
/// let result = example().await;
/// match result {
///     Ok(response) => println!("Fetched news: {:?}", response),
///     Err(e) => eprintln!("Error fetching news: {}", e),
/// }
/// ```
///
pub async fn run(value_config: &ValueConfig) -> Result<MarketAuxResponse, ApiError> {

    // Load the API configuration, including the API key and base URL.
    let config = RequestConfig::new(value_config);

    // Create path parameters for the API request, specifying the endpoint "all".
    let path = PathParams::new(&config, "all".to_string());

    // Construct query parameters for the API request, currently set to None for all optional fields.
    let query = QueryParams::new(&config, 
        None, // Symbols, 
        None, // entity_types, 
        None, // industries, 
        None, // countries, 
        None, // sentiment_gte, 
        None, // sentiment_lte, 
        None, // min_match_score, 
        None, // filter_entities, 
        None, // must_have_entities, 
        None, // group_similar, 
        None, // search, 
        None, // domains, 
        None, // exclude_domains, 
        None, // source_ids, 
        None, // exclude_source_ids, 
        None, // language, 
        None, // published_before, 
        Some(&time_rfc3339_opts(value_config.request.delay_secs).as_str()), // published_after, 
        None, // published_on, 
        None, // sort, 
        None, // sort_order, 
        None, // limit, 
        None); // page

    // Create a new HTTP client for making requests.
    let client = Client::new();

    // Initialize the request manager with the created client.
    let req_manager = RequestManager::new(client);

    // Send a GET request to the Marketaux API and await the result.
    let result = req_manager.get(path, Some(query)).await
        .map_err(|e|  {
            eprintln!("Error during GET request: {}", e); // Log error
            e // Repropagate error
        })?;

    // Print the result of the GET request to the console.
    println!("Result of GET req: {:?}", result);

    // Return the result of the API request.
    Ok(result)
}


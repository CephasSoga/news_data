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
use std::sync::Arc;
use std::time::Duration;
use std::hash::{Hash, Hasher};

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_value};
use tracing::{warn, debug, info, error};
use tokio::sync::Mutex;

use crate::cache::SharedLockedCache;
use crate::config::ValueConfig;
use crate::utils::{get_resp_value_from_cache_or_fetch, time_rfc3339_opts};
use twitter_v2::oauth2::helpers::variant_name;
use crate::options::FetchType;
use crate::errors::{AbstractApiError, ApiError};
use crate::options::MAQueryParams as QueryParams;

const BASE_URL: &str = "https://api.marketaux.com/v1/news";
pub const ALL_NEWS_ENDPOINT: &str = "all";
pub const SIMILAR_NEWS_ENDPOINT: &str = "similar";
pub const NEWS_BY_UUID: &str = "uuid";
const ENDPONT_MAP_KEY: &str = "endpoint";
const API_TOKEN_MAP_KEY: &str = "api_token";
const FETCH_TYPE_KEY_MAP: &str = "fetch_type";


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
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        from_str(json)
    }

    pub fn to_json(&self) -> Result<Value, ApiError> {
        to_value(self).map_err(|err| ApiError::JsonParseError { message: err.to_string()})
    }

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


pub struct MarketAuxApiClient {
    client: Arc<Client>,
    cache: Arc<Mutex<SharedLockedCache>>,
    config: Arc<ValueConfig>,
}
impl MarketAuxApiClient {

    pub fn new(client: Arc<Client>, cache: Arc<Mutex<SharedLockedCache>>, config: Arc<ValueConfig>) -> Self {
        Self {client, cache, config}
    }

    fn append_to_base_url(&self, endpoint: &str) -> String {
        format!("{}/{}", BASE_URL, endpoint)
    }

    async fn get(
        &self,
        fetch_type: &FetchType,
        endpoint: &str,
        query_params: Option<QueryParams>   
    ) -> Result<Value, ApiError> {
        match fetch_type {
            FetchType::MarketAux => {
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

    async fn get_(
        &self,
        endpoint: &str,
        query_params: Option<QueryParams>
    ) -> Result<Value, ApiError> {
            // Send GET request
            let response = self
            .client
            .get(&self.append_to_base_url(endpoint))
            .query(&query_params)
            .send()
            .await.map_err(|e| {
                warn!("MarketAux client encountered an error during GET request.");
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
        // Also the only place the Response super-struct `MarketAuxResponse` is Actually used.
        // For data integrity reasons.
        let response_json: MarketAuxResponse = response.json().await.map_err(|e| {
            error!("Failed to read body: {:?}", e);
            ApiError::JsonParseError { message: e.to_string() }
        })?; // Handle JSON parsing error

        response_json.to_json()
    }

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

    fn insert_api_token(&self, value: Arc<Value>) -> Arc<Value> {
        let mut value = Arc::try_unwrap(value).unwrap_or_else(|v| (*v).clone());
        if let Value::Object(ref mut map) = value {
            map.insert(API_TOKEN_MAP_KEY.to_string(), Value::String(self.config.api.marketaux.clone()));
        }
        Arc::new(value)
    }

    fn pop_endpoint(&self, value: Arc<Value>) -> Option<((String, Value), Arc<Value>)> {
        let mut value = Arc::try_unwrap(value).unwrap_or_else(|v| (*v).clone());
        if let Value::Object(ref mut map) = value {
            Some((map
                    .remove_entry(ENDPONT_MAP_KEY)
                    .unwrap_or((ENDPONT_MAP_KEY.to_string(), Value::String("".to_string()))), Arc::new(value))
            )
        } else {
            None
        }
    }

    pub async fn poll(&self, args: Arc<Value>) -> Result<Value, ApiError> {
        // Insert API token into the provided args value.
        let args = self.insert_api_token(args);
        // Extract the endpoint from the provided args value.
        if let Some(((_key, endpoint), args)) = self.pop_endpoint(args) {
            let endpoint = endpoint.as_str()
                .unwrap_or_else(|| ALL_NEWS_ENDPOINT);
            // Perform GET request with retry mechanism.
            let mut retry_count = 0;
            let max_retries = self.config.task.max_retries;
            let delay_ms = self.config.task.base_delay_ms as u64;
            let delay = Duration::from_millis(delay_ms);
            let fetch_type = args.get(FETCH_TYPE_KEY_MAP) // which does not get popped out of the query params
                .and_then(|s| s.as_str())
                .map(FetchType::from_str)
                .unwrap_or(FetchType::Unknown);
            loop {
                match self.get(&fetch_type, endpoint, Some(QueryParams::try_from(args.clone())?)).await {
                    Ok(response) => {
                        info!("API GET Response was successful? : {:?}", bool::from(!response.is_null()));
                        return Ok(response);
                    }
                    Err(error) => {
                        if retry_count >= max_retries {
                            error!("Failed to fetch data after {} retries.", self.config.task.max_retries);
                            return Err(error);
                        }
                        retry_count += 1;
                        tokio::time::sleep(delay).await;
                        warn!("Attempt {}/{} failed with error: {:?}. Retrying in {} seconds.", retry_count, max_retries, error, delay_ms);
                        debug!("Retrying request due to error: {:?}", error);
                    }
                }
            }
        } else {
            error!("No endpoint found in the provided args value.");
            Err(ApiError::NoEndpointProvided)
        }
    }
}

pub async fn run(endpoint: &str, client: Arc<Client>, cache: Arc<Mutex<SharedLockedCache>>, config: Arc<ValueConfig>) -> Result<Value, ApiError> {
    // Construct query parameters for the API request, currently set to None for all optional fields.
    let query = QueryParams::new(
        &config.api.marketaux, 
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
        Some(&time_rfc3339_opts(config.request.delay_secs).as_str()), // published_after, 
        None, // published_on, 
        None, // sort, 
        None, // sort_order, 
        None, // limit, 
        None); // page

    // Initialize the request manager with the created client.
    let req_manager = MarketAuxApiClient::new(client, cache, config);

    // Send a GET request to the Marketaux API and await the result.
    let result = req_manager.get_(endpoint, Some(query)).await
        .map_err(|e|  {
            error!("Error during GET request: {}", e); // Log error
            e // Repropagate error
        })?;

    // Return the result of the API request.
    Ok(result)
}


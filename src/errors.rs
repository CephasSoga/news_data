
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use std::hash::{Hash, Hasher};

use  thiserror::Error;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};


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
    /// When no endpoint was provided.
    NoEndpointProvided,
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
            ApiError::NoEndpointProvided => {
                write!(f, "No endpoint provided")
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


#[derive(Debug, Error)]
pub enum FMPApiError {
    #[error("Failed to fetch data: {0}")]
    FetchError(String),
    
    #[error("Task encountered an error: {0}")]
    TaskError(String),
    
    #[error("Failed to parse data: {0}")]
    ParseError(String),
}
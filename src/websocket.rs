#![allow(dead_code)]
#![allow(warnings)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::pin::Pin;

use futures_util::{SinkExt, StreamExt, Future};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
//use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use async_tungstenite::tokio::accept_async_with_config;
use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::tungstenite::error::Error;
use tungstenite::protocol::WebSocketConfig;
use tokio::net::lookup_host;
use serde_json::{to_value, from_str, Value};
use serde::{Serialize, Deserialize};
use tracing::{error, info, warn};
use reqwest::Client;

use crate::logging::{LogLevel, Logger, setup_logger};
use crate::config::ValueConfig;
use crate::cache::SharedLockedCache;
use crate::fmp::FMPClient;
use crate::alphavantage::{AlphaVantageApiClient, BASE_FUNCTION};
use crate::marketaux::{MarketAuxApiClient, ALL_NEWS_ENDPOINT, SIMILAR_NEWS_ENDPOINT, NEWS_BY_UUID};
use crate::request::HTTPClient;
use crate::request_parser::parser::CallParser;
use crate::request_parser::params::*;

const REQUEST_SUCCUESS: u32 = 200;
const REQUEST_FAILED: u32 = 400;
const NOT_ALLOWED: u32 = 500;
const REQUEST_TIMEOUT: u32 = 408;
const REQUEST_CANCELED: u32 = 499;
const REQUEST_INTERNAL_ERROR: u32 = 503;
const NOT_FOUND: u32 = 404;     
const REQUEST_RATE_LIMITED: u32 = 429;
const CACHE_SIZE: usize = 1000;

enum Outcome {
    Failure,
    NotAllowed,
    Timeout,
    Canceled,
    InternalError,
    NotFound,
    RateLimited,
}

pub struct ServerSocket {
    address: String,
    make: MakeResponse,
    state: Arc<PollState>,
}
impl ServerSocket {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            make: MakeResponse::new(),
            state: Arc::new(PollState::default()),
        }
    }


    pub async fn run(&mut self) -> Result<(), Error> {
        info!(message="Resolving address", addr=self.address);
        let mut addrs = lookup_host(&self.address).await
            .map_err(|e| println!("Error resolving address: {}", e.to_string()))
            .unwrap();

        let addr = addrs.next().ok_or_else(|| {
            error!(err="Failed to resolve address", addr=self.address);
            Error::Url(tungstenite::error::UrlError::NoHostName)
        })?;

        info!("Setting address: {}", self.address);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| println!("Error: {}", e.to_string()))
            .unwrap();

        info!("Building RMake...");
        let _ = self.make.build();

        println!("WebSocket server listening on: {}", self.address);

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New connection from: {}", addr);
            tokio::spawn(Self::handle_connection(stream, self.make.clone(), self.state.clone()));
        }

        Ok(())
    }

    async fn handle_connection(stream: TcpStream, make: MakeResponse, state: Arc<PollState>) {
        let config = Some(WebSocketConfig::default());


        let ws_stream = match accept_async_with_config(stream, config).await {
            Ok(ws_stream) => ws_stream,
            Err(e) => {
                error!("Error during handshake: {}", e);
                return;
            }
        };

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<String>(100);

        // Spawn task to handle outgoing messages
        let write_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(_json) => {
                            let state = Arc::clone(&state);
                            info!("Making Response...");
                            let response = make.make(state, &text).await;
                            info!("Sending response...");
                            if let Err(_) = tx.send(format!("{}", &response)).await {
                                break;
                            }
                            info!("Response sent.");
                        }
                        Err(e) => {
                            error!("Failed to parse JSON: {}", e);
                            if let Err(_) = tx.send("Invalid JSON".to_string()).await {
                                break;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    warn!("Error receiving message: {}", e);
                    break;
                }
                _ => {}
            }
        }

        write_task.abort();
    }
}

pub struct PollState {
    http_client: Arc<HTTPClient>,
    client: Arc<Client>,
    cache: Arc<Mutex<SharedLockedCache>>,
    config: Arc<ValueConfig>,
}
impl Default for PollState{
    fn default() -> Self {
        Self {
            http_client: Arc::new(HTTPClient::new().unwrap()),
            client: Arc::new(Client::new()),
            cache: Arc::new(Mutex::new(SharedLockedCache::new(CACHE_SIZE))),
            config: Arc::new(ValueConfig::new().unwrap()),   
        }
    }
}
struct Collection;
impl Collection {
    async fn get_news_from_alphavantage_unpinned(state: Arc<PollState>, args: Arc<Value>) -> Value {
        let alphavantage_client = AlphaVantageApiClient::new(
            state.client.clone(),
            state.cache.clone(),
            state.config.clone(),
        );
        match alphavantage_client.poll(args).await {
            Ok(v) => v,
            Err(e) => Value::String(format!("AlphaVantage Client polling failed: {}", e)),
        }
    }

    async fn get_news_from_marketaux_unpinned(state: Arc<PollState>, args: Arc<Value>) -> Value {
        let marketaux_client = MarketAuxApiClient::new(
            state.client.clone(),
            state.cache.clone(),
            state.config.clone(),
        );

        match marketaux_client.poll(args).await {
            Ok(v) => v,
            Err(e) => Value::String(format!("MarketAux Client polling failed: {}", e)),
        }
    }

    async fn get_news_from_fmp_unpinned(state: Arc<PollState>, args: Arc<Value>) -> Value {
        let fmp_client = FMPClient::new(
            state.http_client.clone(),
            state.cache.clone(),
            state.config.clone(),
        );

        match fmp_client.poll(args).await {
            Ok(v) => v,
            Err(e) => Value::String(format!("FMP Client polling failed: {}", e)),
        }
    }

    fn alphvantage_func(
        state: Arc<PollState>,
        args: Arc<Value>,
    ) -> Pin<Box<dyn Future<Output = Value> + Send + 'static>> {
        Box::pin(async move {
            Collection::get_news_from_alphavantage_unpinned(state, args).await
        })
    }

     fn marketaux_func(
        state: Arc<PollState>,
        args: Arc<Value>,
    ) -> Pin<Box<dyn Future<Output = Value> + Send + 'static>> {
        Box::pin(async move {
            Collection::get_news_from_marketaux_unpinned(state, args).await
        })
    }

    fn fmp_func(
        state: Arc<PollState>,
        args: Arc<Value>,
    ) -> Pin<Box<dyn Future<Output = Value> + Send + 'static>> {
        Box::pin(async move {
            Collection::get_news_from_fmp_unpinned(state, args).await
        })
    }
}


type Func = fn(Arc<PollState>, Arc<Value>) -> Pin<Box<dyn Future<Output = Value> + Send + 'static>>;

#[derive(Clone)]
pub struct MakeResponse{
    fn_map: HashMap<String, Box<Func>>,
}
impl MakeResponse {
    pub fn new() -> Self {
        Self {
            fn_map: HashMap::new(),
        }
    }

    fn register_function(&mut self, where_: String, func: Func) {
        self.fn_map.insert(where_, Box::new(func));
    }

    pub fn build(&mut self) {
        self.register_function("alphavantage_news_polling".to_string(), Collection::alphvantage_func);
        self.register_function("marketaux_news_polling".to_string(), Collection::marketaux_func);
        self.register_function("fmp_news_polling".to_string(), Collection::fmp_func);
    }

    pub async fn make(&self, state: Arc<PollState>, s: &str) -> Value {
        println!("Parsing request...");
        let call_request = match CallParser::key_lookup_parse_json(s) {
            Ok(req) => req,
            Err(err) => return self.return_error(Outcome::Failure, err),
        };
    
        if call_request.target.to_str() == "task" {
            if let Some(task_args) = call_request.args.for_task {
                if let TaskFunction::AggregatedPolling = task_args.function {
                    return self.handle_task(state, task_args).await;
                }
            }
        }
    
        self.return_error(Outcome::NotAllowed, "Invalid request".to_string())
    }
    async fn handle_task(&self, state: Arc<PollState>, task_args: TaskArgs) -> Value {
        let where_ = task_args.look_for.where_;
        info!("Extracting Args...");
        if let Some(args) = task_args.params {
            info!("Executing task function: {}", &where_);
            if let Some(func) = self.map_func(&where_) {
                let args = Arc::new(to_value(args).unwrap());
                let result = func(state, args).await;
                return self.return_success(result);
            } else {
                error!("Invalid task function: {}", &where_);
                return self.return_error(Outcome::Failure, format!("Invalid task function: {}", &where_));
            }
        }
    
        self.return_error(Outcome::Failure, "Invalid task arguments".to_string())
    }
    
    fn map_func(&self, where_: &String) -> Option<Box<Func>> {
        if let Some(func) = self.fn_map.get(where_).cloned() {
            Some(func.clone())
        } else {
            None
        }
    }

    async fn exec_func(&self, func: &Func, state: Arc<PollState>, args: Arc<Value>) -> Result<Value, Error> {
        let result = func(state, args).await;
        Ok(result)
    }

    fn return_success(&self, message: Value) -> Value {
        ServerResponse::new(REQUEST_SUCCUESS, Some(message), None).to_json()
    }

    fn return_error(&self, outcome: Outcome, reason: String) -> Value {
        let status = match outcome {
            Outcome::Failure => REQUEST_FAILED,
            Outcome::Canceled => REQUEST_CANCELED,
            Outcome::Timeout => REQUEST_TIMEOUT,
            Outcome::NotAllowed => NOT_ALLOWED,
            Outcome::NotFound => NOT_FOUND,
            Outcome::RateLimited=> REQUEST_RATE_LIMITED,
            Outcome::InternalError => REQUEST_INTERNAL_ERROR,
        };
        ServerResponse::new(status, None, Some(reason)).to_json()

    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub status: u32,
    pub message: Option<Value>,
    pub reason: Option<String>,  // Only for failed requests
}
impl ServerResponse {
    pub fn new(status: u32, message: Option<Value>, reason: Option<String>) -> Self {
        Self {
            status,
            message,
            reason,
        }
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    
}

////
pub async fn run() -> Result<(), Error> {
    let mut server = ServerSocket::new("0.0.0.0:8080");
    server.run().await
}
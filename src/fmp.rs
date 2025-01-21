
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use futures::future::OptionFuture;
use serde_json::{Value, from_str};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tracing_subscriber::field::debug; 
use thiserror::Error;
use tracing::info;

use crate::config::ValueConfig;
use crate::cache::SharedLockedCache;
use crate::request::HTTPClient;
use crate::options::FetchType;
use crate::server_types::{FMPArticle, FMPMarketSentiment};
use crate::utils::{retry, get_from_cache_or_fetch};

const FMP_ARTICLES_V3: &str = "fmp/articles";
const GENERAL_NEWS_V4: &str = "general_news";
const STOCK_NEWS_V3: &str = "stock_news";
const STOCK_RSS_V4: &str = "stock-news-sentiments-rss-feed";
const FOREX_NEWS_V4: &str = "forex_news";
const CRYPTO_NEWS_V4: &str = "crypto_news";
const PRESS_RELEASES_V3: &str = "press_releases";
const HISTORICAL_SOCIAL_SENTIMENT_V4: &str = "historical/social-sentiment";
const TRENDING_SOCIAL_SENTIMENT_V4: &str = "social-sentiments/trending";
const SOCIAL_SENTIMENT_CHANGES_V4: &str = "social-sentiments/change";


#[derive(Debug, Error)]
pub enum FMPApiError {
    #[error("Failed to fetch data: {0}")]
    FetchError(String),
    
    #[error("Task encountered an error: {0}")]
    TaskError(String),
    
    #[error("Failed to parse data: {0}")]
    ParseError(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    News(Vec<FMPArticle>),
    MarketSentiment(Vec<FMPMarketSentiment>),
}
impl From<Value> for Content {
    fn from(value: Value) -> Content {
        if let Ok(news) = serde_json::from_value::<Vec<FMPArticle>>(value.clone()) {
            Content::News(news)
        } else if let Ok(market_sentiment) = serde_json::from_value::<Vec<FMPMarketSentiment>>(value) {
            Content::MarketSentiment(market_sentiment)
        } else {
            panic!("Failed to parse Content from Value");
        } 
    }
}

pub enum AbstactContent {
    News,
    MarketSentiment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    sorted: Option<bool>,
    unsorted: Option<bool>,
    empty: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pageable {
    sort: Sort,
    page_size: Option<u64>,
    page_number: Option<u64>,
    offset: Option<u64>,
    paged: Option<bool>,
    unpaged: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FMPApiResponse {
    content: Option<Content>,
    pageable: Option<Pageable>,
    total_pages: Option<u64>,
    total_elements: Option<u64>,
    last: Option<bool>,
    number: Option<u64>,
    size: Option<u64>,
    number_of_elements: Option<u64>,
    sort: Option<Sort>,
    first: Option<bool>,
    empty: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryParams {
    /// Symbol. E.g: AAPL.
    symbol: Option<String>,

    /// A string lis t of tickers. E.g: AAPL,FB
    tickers: Option<String>,

    /// Date in YYYY-MM-DD format.
    from: Option<String>,

    /// Date in YYYY-MM-DD format.
    to: Option<String>,

    /// Limit the number of pages. Default is 1.
    page: Option<u64>,

    /// Limit the number of results per page. Default is 10.
    size: Option<u64>,

    /// `bullish` or `bearish`.
    type_name: Option<String>,

    /// `stockwits`
    source: Option<String>,
}
impl Into<Option<Vec<(String, String)>>> for QueryParams {
    fn into(self) -> Option<Vec<(String, String)>> {
        let mut query_params: Vec<(String, String)> = Vec::new();
        if let Some(symbol) = &self.symbol {
            query_params.push(("symbol".to_string(), symbol.to_string()));
        }
        if let Some(tickers) = &self.tickers {
            query_params.push(("tickers".to_string(), tickers.to_string()));
        }
        if let Some(from) = &self.from {
            query_params.push(("from".to_string(), from.to_string()));
        }
        if let Some(to) = &self.to {
            query_params.push(("to".to_string(), to.to_string()));
        }
        if let Some(page) = &self.page {
            query_params.push(("page".to_string(), page.to_string()));
        }
        if let Some(size) = &self.size {
            query_params.push(("size".to_string(), size.to_string()));
        }
        if let Some(type_name) = &self.type_name {
            query_params.push(("type_name".to_string(), type_name.to_string()));
        }
        if let Some(source) = &self.source {
            query_params.push(("source".to_string(), source.to_string()));
        }
        match query_params.len() {
            0 => None,
            _ => Some(query_params),
        }

    }
}

impl From<Value> for QueryParams {
    fn from(value: Value) -> Self {
        QueryParams {
            symbol: value.get("symbol").and_then(|v| v.as_str().map(|s| s.to_string())),
            tickers: value.get("tickers").and_then(|v| v.as_str().map(|s| s.to_string())),
            from: value.get("from").and_then(|v| v.as_str().map(|s| s.to_string())),
            to: value.get("to").and_then(|v| v.as_str().map(|s| s.to_string())),
            page: value.get("page").and_then(|v| v.as_u64()),
            size: value.get("size").and_then(|v| v.as_u64()),
            type_name: value.get("type_name").and_then(|v| v.as_str().map(|s| s.to_string())),
            source: value.get("source").and_then(|v| v.as_str().map(|s| s.to_string())),
        }
    }
}

impl Display for  QueryParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }   
}

pub struct FMPClient{
    http_client: Arc<HTTPClient>,
    cache: Arc<Mutex<SharedLockedCache>>,
    config: Arc<ValueConfig>,
}
impl FMPClient {
    pub fn new(http_client: Arc<HTTPClient>, cache: Arc<Mutex<SharedLockedCache>>, config: Arc<ValueConfig>) -> Self {
        FMPClient {
            http_client,
            cache,
            config
        }
    }

    async fn get_fmp_articles(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("fmp_articles_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v3(FMP_ARTICLES_V3,query_params.into()).await
            }, 
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_general_news(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key  = format!("general_news_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(GENERAL_NEWS_V4, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_stock_news(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("stock_news_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v3(STOCK_NEWS_V3, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async  fn get_stock_rss(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("stock_rss_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(STOCK_RSS_V4, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_forex_news(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("forex_news_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(FOREX_NEWS_V4, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_crypto_news(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("crypto_news_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(CRYPTO_NEWS_V4, query_params.into()).await
                },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_press_releases(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("press_releases_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v3(PRESS_RELEASES_V3, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_historical_social_sentiment(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("historical_social_sentiment_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(HISTORICAL_SOCIAL_SENTIMENT_V4, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_trending_social_sentiment(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("trending_social_sentiment_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(TRENDING_SOCIAL_SENTIMENT_V4, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn get_social_sentiment_changes(&self, query_params: QueryParams) -> Result<Value, FMPApiError> {
        let key = format!("social_sentiment_changes_{}", &query_params);
        get_from_cache_or_fetch(
            &self.cache, 
            &key, 
            || async {
                self.http_client.get_v4(SOCIAL_SENTIMENT_CHANGES_V4, query_params.into()).await
            },
            self.config.task.cache_ttl
        ).await
        .map_err(|e| FMPApiError::FetchError(e.to_string()))
    }

    async fn fetch(&self, fetch_type: FetchType, query_params: QueryParams) -> Result<FMPApiResponse, FMPApiError> {
        match fetch_type {
            FetchType::FMPArticle => {
                let result = self.get_fmp_articles(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            },
            FetchType::GeneralNews => {
                let result = self.get_general_news(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }
            FetchType::StockNews => {
                let result = self.get_stock_news(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            },
            FetchType::StockRSS => {
                let result = self.get_stock_rss(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }
            FetchType::ForexNews => {
                let result = self.get_forex_news(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }
            FetchType::CryptoNews => {
                let result = self.get_crypto_news(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }
            FetchType::PressReleases => {
                let result = self.get_press_releases(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::News)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }

            FetchType::SocialSentimentHistory => {
                let result = self.get_historical_social_sentiment(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::MarketSentiment)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }
            FetchType::SocialSentimentTrending => {
                let result = self.get_trending_social_sentiment(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::MarketSentiment)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }
            FetchType::SocialSentimentChanges => {
                let result = self.get_social_sentiment_changes(query_params).await?;
                let articles: FMPApiResponse = self.response_from_value(result, AbstactContent::MarketSentiment)
                    .map_err(|e| FMPApiError::ParseError(e.to_string()))?;
                Ok(articles)
            }

            _ => Err(FMPApiError::TaskError(format!("Fetch type `{}` is not supported.", fetch_type))),
        }
    }

    fn response_from_value(&self, value: Value, abstract_type: AbstactContent) -> Result<FMPApiResponse, FMPApiError> {
        let content = match abstract_type {
            AbstactContent::News => {
                let content_value = value.get("content");
                content_value.and_then(|v| {
                    let result: Result<Vec<FMPArticle>, _> = serde_json::from_value(v.clone());
                    result.map(Content::News).ok()
                })
            }
            AbstactContent::MarketSentiment => {
                let content_value = value.get("content");
                content_value.and_then(|v| {
                    let result: Result<Vec<FMPMarketSentiment>, _> = serde_json::from_value(v.clone());
                    result.map(Content::MarketSentiment).ok()
                })
            }
        };

        let pageable = value.get("pageable").and_then(|v| serde_json::from_value(v.clone()).ok());
        let total_pages = value.get("totalPages").and_then(|v| v.as_u64());
        let total_elements = value.get("totalElements").and_then(|v| v.as_u64());
        let last = value.get("last").and_then(|v| v.as_bool());
        let number = value.get("number").and_then(|v| v.as_u64());
        let size = value.get("size").and_then(|v| v.as_u64());
        let number_of_elements = value.get("numberOfElements").and_then(|v| v.as_u64());
        let sort = value.get("sort").and_then(|v| serde_json::from_value(v.clone()).ok());
        let first = value.get("first").and_then(|v| v.as_bool());
        let empty = value.get("empty").and_then(|v| v.as_bool());
    
        Ok(FMPApiResponse {
            content,
            pageable,
            total_pages,
            total_elements,
            last,
            number,
            size,
            number_of_elements,
            sort,
            first,
            empty,
        })
    }

    pub async fn poll(&self, args: Value) -> Result<FMPApiResponse, FMPApiError> {
        let query_params = QueryParams::from(args.clone());
        let fetch_type = FetchType::from(args);
        retry(
            &self.config.clone(), 
            || async {
                self.fetch(fetch_type.clone(), query_params.clone()).await
            }).await
    }
}
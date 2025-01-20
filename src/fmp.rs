
use std::sync::Arc;

use serde_json::{Value, from_str}; 

use crate::request::HTTPClient;
use crate::options::FetchType;
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

pub struct FMPClient{
    http_client: Arc<HTTPClient>
}
impl FMPClient {
    pub fn new(http_client: Arc<HTTPClient>) -> Self {
        FMPClient {
            http_client,
        }
    }

    async fn get_fmp_articles(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v3(FMP_ARTICLES_V3,query_params.into()).await
    }

    async fn get_general_news(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(GENERAL_NEWS_V4, query_params.into()).await
    }

    async fn get_stock_news(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v3(STOCK_NEWS_V3, query_params.into()).await
    }

    async  fn get_stock_rss(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(STOCK_RSS_V4, query_params.into()).await
    }

    async fn get_forex_news(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(FOREX_NEWS_V4, query_params.into()).await
    }

    async fn get_crypto_news(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(CRYPTO_NEWS_V4, query_params.into()).await
    }

    async fn get_press_releases(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v3(PRESS_RELEASES_V3, query_params.into()).await
    }

    async fn get_historical_social_sentiment(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(HISTORICAL_SOCIAL_SENTIMENT_V4, query_params.into()).await
    }

    async fn get_trending_social_sentiment(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(TRENDING_SOCIAL_SENTIMENT_V4, query_params.into()).await
    }

    async fn get_social_sentiment_changes(&self, query_params: QueryParams) -> Result<Value, reqwest::Error> {
        self.http_client.get_v4(SOCIAL_SENTIMENT_CHANGES_V4, query_params.into()).await
    }
}
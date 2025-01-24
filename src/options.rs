
use std::sync::Arc;
use std::fmt::Display;
use std::fmt;
use std::time::Duration;
use std::hash::{Hash, Hasher};

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, to_value};
use tokio::sync::Mutex;

use crate::errors::ApiError;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchType {
    MarketAux,
    AlphaVantage,
    FMPArticle,
    GeneralNews,
    StockNews,
    StockRSS,
    CryptoNews,
    ForexNews,
    PressReleases,
    SocialSentimentHistory,
    SocialSentimentTrending,
    SocialSentimentChanges,
    Unknown
}
impl Display for  FetchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FetchType::MarketAux => "Market Auxiliary",
            FetchType::AlphaVantage => "Alpha Vantage",
            FetchType::FMPArticle => "FMP Article",
            FetchType::GeneralNews => "General News",
            FetchType::StockNews => "Stock News",
            FetchType::StockRSS => "Stock RSS",
            FetchType::CryptoNews => "Crypto News",
            FetchType::ForexNews => "Forex News",
            FetchType::PressReleases => "Press Releases",
            FetchType::SocialSentimentHistory => "Social Sentiment History",
            FetchType::SocialSentimentTrending => "Social Sentiment Trending",
            FetchType::SocialSentimentChanges => "Social Sentiment Changes",
            _ => "Unknown",
        };
        write!(f, "{}", name)
    }
}

impl FetchType {
    pub fn from(value: Arc<serde_json::Value>) -> FetchType {

        let value = Arc::try_unwrap(value).unwrap_or_else(|v| (*v).clone());
        match value["function"].as_str() {
            Some("marketaux") => FetchType::MarketAux,
            Some("alphavantage") => FetchType::AlphaVantage,
            Some("fmp articles") => FetchType::FMPArticle,
            Some("general news") => FetchType::GeneralNews,
            Some("stock news") => FetchType::StockNews,
            Some("stock rss") => FetchType::StockRSS,
            Some("crypto news") => FetchType::CryptoNews,
            Some("forex news") => FetchType::ForexNews,
            Some("press releases") => FetchType::PressReleases,
            Some("social sentiment history") => FetchType::SocialSentimentHistory,
            Some("social sentiment trending") => FetchType::SocialSentimentTrending,
            Some("social sentiment changes") => FetchType::SocialSentimentChanges,
            _ => FetchType::Unknown,
        }
    
    }

    pub fn from_str(s: &str) -> FetchType {
        match s {
            "marketaux" => FetchType::MarketAux,
            "alphavantage" => FetchType::AlphaVantage,
            "fmp_articles" => FetchType::FMPArticle,
            "general_news" => FetchType::GeneralNews,
            "stock_news" => FetchType::StockNews,
            "stock_rss" => FetchType::StockRSS,
            "crypto_news" => FetchType::CryptoNews,
            "forex_news" => FetchType::ForexNews,
            "press_releases" => FetchType::PressReleases,
            "social_sentiment_history" => FetchType::SocialSentimentHistory,
            "social_sentiment_trending" => FetchType::SocialSentimentTrending,
            "social_sentiment_changes" => FetchType::SocialSentimentChanges,
            _ => FetchType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AVQueryParams {
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

impl AVQueryParams {
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
impl TryFrom<Value> for AVQueryParams {
    type Error = ApiError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|err| ApiError::JsonParseError { message: err.to_string() })
    }    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents the HTTP request parameters for the Marketaux API.
///
/// This struct contains all the parameters that can be used to customize the API request
/// to fetch financial news articles. Each field corresponds to a specific query parameter
/// that can be included in the request.
pub struct MAQueryParams {
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

impl MAQueryParams {
    /// Creates a new instance of QueryParams with required and optional parameters.
    pub fn new(
        apikey: &str,
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
            api_token: apikey.to_string(),
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
}
impl TryFrom<Value> for MAQueryParams {
    type Error = ApiError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|err| ApiError::JsonParseError { message: err.to_string() })
    }    
}
impl TryFrom<Arc<Value>> for MAQueryParams {
    type Error = ApiError;
    fn try_from(value: Arc<Value>) -> Result<Self, Self::Error> {
        // Unwrap the Arc to get the inner Value
        let value = Arc::try_unwrap(value).unwrap_or_else(|v| (*v).clone());
        serde_json::from_value(value).map_err(|err| ApiError::JsonParseError { message: err.to_string() })
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FMPQueryParams {
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
impl Into<Option<Vec<(String, String)>>> for FMPQueryParams {
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

impl From<Value> for FMPQueryParams {
    fn from(value: Value) -> Self {
        FMPQueryParams {
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

impl From<Arc<Value>> for FMPQueryParams {
    fn from(value: Arc<Value>) -> Self {
        let value = Arc::try_unwrap(value).unwrap_or_else(|v| (*v).clone());
        FMPQueryParams::from(value.clone())
    }
}

impl Display for  FMPQueryParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }   
}

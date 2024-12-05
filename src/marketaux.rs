
//! ## A Rust wrapper of the [Marketaux API](https://www.marketaux.com).
//! 
//! Get all the latest global financial news and filter by entities identified within articles to build concise news feeds. 
//! Also provided is analysis of each entity identified in articles. Note that not every article may have entities identified. 
//! To retrieve all news for articles with identified entities, use the parameter must_have_entities, 
//! or specify any of the entity params such as symbols or exchanges as defined below to produce more concise results.
//! 
//! ## Reference:
//! [Official Marketaux Documentation](https://www.marketaux.com/documentation).

// API token (Required): Your API token which can be found on your account dashboard.
// Example: api_token=your_api_token

// symbols (Optional): Specify entity symbol(s) identified within the article.
// Example: symbols=TSLA,AMZN,MSFT

// entity_types (Optional): Specify the type of entities identified within the article.
// Example: entity_types=index,equity

// industries (Optional): Specify the industries of entities identified within the article.
// Example: industries=Technology,Industrials

// countries (Optional): Specify the country of the exchange for identified entities within the article.
// Example: countries=us,ca

// sentiment_gte (Optional): Find articles with entities having a sentiment score greater than or equal to x.
// Example: sentiment_gte=0 - Finds articles that are neutral or positive

// sentiment_lte (Optional): Find articles with entities having a sentiment score less than or equal to x.
// Example: sentiment_lte=0 - Finds articles that are neutral or negative

// min_match_score (Optional): Find articles with entities having a match score greater than or equal to min_match_score.

// filter_entities (Optional): By default, all entities for each article are returned. 
// Set this to true to return only relevant entities for your query.
// Example: filter_entities=true (Only relevant entities will be returned)

// must_have_entities (Optional): Set to true to ensure at least one entity is identified within the article.
// Example: must_have_entities=true

// group_similar (Optional): Group similar articles to avoid displaying multiple articles on the same topic/subject. Default is true.
// Example: group_similar=true

// search (Optional): Use to search for specific terms or phrases in articles. 
// Supports advanced query usage with operators (+, |, -, ", *, ( ) )
// Example: search="ipo" -nyse (Searches for articles mentioning "ipo" but not NYSE)

// domains (Optional): Specify a comma-separated list of domains to include in the search.
// Example: domains=adweek.com,adage.com

// exclude_domains (Optional): Specify a comma-separated list of domains to exclude from the search.
// Example: exclude_domains=example.com

// source_ids (Optional): Specify a comma-separated list of source IDs to include in the search.
// Example: source_ids=adweek.com-1,adage.com-1

// exclude_source_ids (Optional): Specify a comma-separated list of source IDs to exclude from the search.

// language (Optional): Specify a comma-separated list of languages to include. Default is all languages.
// Example: language=en,es (Includes English and Spanish articles)

// published_before (Optional): Find articles published before the specified date.
// Example: published_before=2024-12-05T08:25:06

// published_after (Optional): Find articles published after the specified date.
// Example: published_after=2024-12-05T08:25:06

// published_on (Optional): Find articles published on the specified date.
// Example: published_on=2024-12-05

// sort (Optional): Sort articles by published date, entity match score, entity sentiment score, or relevance score.
// Example: sort=entity_match_score

// sort_order (Optional): Specify the sort order for the sort parameter. Options: "desc" | "asc" (Default is "desc").
// Example: sort_order=asc

// limit (Optional): Specify the number of articles to return. Default is the maximum specified for your plan.
// Example: limit=50

// page (Optional): Use for pagination to navigate through the result set. Default is 1.
// Example: page=2



use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
/// Wrapper of the Marketaux API response.
/// 
/// [See example here](https://www.marketaux.com/documentation).
pub struct MarketAuxResponse {
    pub meta: Meta,
    pub data: Vec<NewsItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub found: i64,
    pub returned: i64,
    pub limit: i64,
    pub page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsItem {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub keywords: String,
    pub snippet: String,
    pub url: String,
    pub image_url: String,
    pub language: String,
    #[serde(rename = "published_at")]
    pub published_at: String, // you can change this to DateTime if needed
    pub source: String,
    pub relevance_score: Option<f64>,
    pub entities: Vec<Entity>,
    pub similar: Vec<Value>, // Assuming similar items can vary in structure
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub symbol: String,
    pub name: String,
    pub exchange: Option<String>,
    pub exchange_long: Option<String>,
    pub country: String,
    pub r#type: String, // Using `r#type` to avoid conflicting with the `type` keyword
    pub industry: String,
    pub match_score: f64,
    pub sentiment_score: f64,
    pub highlights: Vec<Highlight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub highlight: String,
    pub sentiment: f64,
    #[serde(rename = "highlighted_in")]
    pub highlighted_in: String,
}


pub struct RequestManager {}
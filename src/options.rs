
use std::fmt::Display;

#[derive(Debug, Clone)]
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
    pub fn from(args: serde_json::Value) -> FetchType {
        match args["function"].as_str() {
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
            "Market Auxiliary" => FetchType::MarketAux,
            "Alpha Vantage" => FetchType::AlphaVantage,
            "FMP Article" => FetchType::FMPArticle,
            "General News" => FetchType::GeneralNews,
            "Stock News" => FetchType::StockNews,
            "Stock RSS" => FetchType::StockRSS,
            "Crypto News" => FetchType::CryptoNews,
            "Forex News" => FetchType::ForexNews,
            "Press Releases" => FetchType::PressReleases,
            "Social Sentiment History" => FetchType::SocialSentimentHistory,
            "Social Sentiment Trending" => FetchType::SocialSentimentTrending,
            "Social Sentiment Changes" => FetchType::SocialSentimentChanges,
            _ => FetchType::Unknown,
        }
    }
}
enum FMPNewsType {
    Crypto,
    Forex,
    Stock,
}

enum TickersListFormat {
    S(String),
    A(Vec<String>)
}

type HtmlLikeString = String;
type UrlString = String;
type DateString = String;

struct FMPArticle {
    title: Option<String>,
    date: Option<String>,
	content: Option<HtmlLikeString>, // html-like string //<p><a href='https://financialmodelingprep.com/financial-summary/SEDG'>SolarEdge Technologies (NASDAQ:SEDG)</a> shares plunged more than 25% intra-day today following the company's preliminary Q3 financial results. Revenue for the quarter is now expected to be between $720 million and $730 million, a significant drop from the earlier projection of $880 million to $920 million,....",
	tickers: Option<TickersListFormat>,
	image: Option<UrlString>,
	link: Option<UrlString>,
	author: Option<String>,
    site: Option<String>,
    published_date: Option<DateString>,
	url: Option<UrlString>,
    symbol: Option<String>,
	text: Option<String>,
    sentiment: Option<String>,
	sentiment_score: Option<f64>,
	updated_at: Option<DateString>,
	created_at: Option<DateString>,
	type_name: Option<FMPNewsType>,

}
impl FMPArticle {
    fn from_value(value: serde_json::Value) -> FMPArticle {
        FMPArticle {
            title: value.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
            date: value.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
            content: value.get("content").and_then(|v| v.as_str()).map(|s| s.to_string()),
            tickers: value.get("tickers").and_then(|v| v.as_array()).map(|a| TickersListFormat::A(a.iter().map(|v| v.as_str().unwrap().to_string()).collect())),
            image: value.get("image").and_then(|v| v.as_str()).map(|s| s.to_string()),
            link: value.get("link").and_then(|v| v.as_str()).map(|s| s.to_string()),
            author: value.get("author").and_then(|v| v.as_str()).map(|s| s.to_string()),
            site: value.get("site").and_then(|v| v.as_str()).map(|s| s.to_string()),
            published_date: value.get("published_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
            url: value.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            symbol: value.get("symbol").and_then(|v| v.as_str()).map(|s| s.to_string()),
            text: value.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()),
            sentiment: value.get("sentiment").and_then(|v| v.as_str()).map(|s| s.to_string()),
            sentiment_score: value.get("sentiment_score").and_then(|v| v.as_f64()),
            updated_at: value.get("updated_at").and_then(|v| v.as_str()).map(|s| s.to_string()),
            created_at: value.get("created_at").and_then(|v| v.as_str()).map(|s| s.to_string()),
            type_name: value.get("type_name").and_then(|v| v.as_str()).map(|s| match s {
                "crypto" => FMPNewsType::Crypto,
                "forex" => FMPNewsType::Forex,
                "stock" => FMPNewsType::Stock,
                _ => panic!("Invalid type_name: {}", s),
            }),
        }
    }
}

struct MarketSentiment {
		date: Option<String>,
		symbol: Option<String>,
		stock_twits_posts: Option<u64>,
		twitter_posts: Option<u64>,
		stock_twits_comments:Option<u64>,
		twitter_comments: Option<u64>,
		stocktwits_likes: Option<u64>,
		twitter_likes: Option<u64>,
		stock_twits_impressions: Option<u64>,
		twitter_impressions: Option<u64>,
		stock_twits_sentiment: Option<f64>,
		twitter_sentiment: Option<f64>,
		name: Option<String>,
		rank: Option<u64>,
		sentiment: Option<u64>,
		last_sentiment: Option<f64>,
		sentiment_change: Option<f64>

}
impl MarketSentiment {
    fn from_value(value: serde_json::Value) -> MarketSentiment {
        MarketSentiment {
            date: value.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
            symbol: value.get("symbol").and_then(|v| v.as_str()).map(|s| s.to_string()),
            stock_twits_posts: value.get("stock_twits_posts").and_then(|v| v.as_u64()),
            twitter_posts: value.get("twitter_posts").and_then(|v| v.as_u64()),
            stock_twits_comments: value.get("stock_twits_comments").and_then(|v| v.as_u64()),
            twitter_comments: value.get("twitter_comments").and_then(|v| v.as_u64()),
            stocktwits_likes: value.get("stocktwits_likes").and_then(|v| v.as_u64()),
            twitter_likes: value.get("twitter_likes").and_then(|v| v.as_u64()),
            stock_twits_impressions: value.get("stock_twits_impressions").and_then(|v| v.as_u64()),
            twitter_impressions: value.get("twitter_impressions").and_then(|v| v.as_u64()),
            stock_twits_sentiment: value.get("stock_twits_sentiment").and_then(|v| v.as_f64()),
            twitter_sentiment: value.get("twitter_sentiment").and_then(|v| v.as_f64()),
            name: value.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            rank: value.get("rank").and_then(|v| v.as_u64()),
            sentiment: value.get("sentiment").and_then(|v| v.as_u64()),
            last_sentiment: value.get("last_sentiment").and_then(|v| v.as_f64()),
            sentiment_change: value.get("sentiment_change").and_then(|v| v.as_f64()),
        
        }
    }
}
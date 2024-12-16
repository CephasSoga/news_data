
use mongodb::bson::doc;


#[allow(dead_code)]

pub mod marketaux;
pub mod alphavantage;
pub mod db;
pub mod config;

#[tokio::main]
async fn main() -> Result<(), Box<db::OpError>> {
    // read config file
    println!("\nReading configurations...");
    let value_config  = config::ValueConfig::new().expect("failed to read config file.");

    // create db_client
    println!("\nIntializing client...");
    let client_wrapper = db::ClientManager::new(&value_config).await
        .map_err(|e| {
            println!("An error occured with client creation");
            e
        })?;
    let client = client_wrapper.get_client();

    let ops = db::DatabaseOps::new(client, "hello", "world");

    let docs = vec![
        doc! {"name": "alice", "age": 24},
        doc! {"name": "john", "age": 22}

    ];

    println!("\nInserting docs...");
    match ops.insert_many(docs)
    .await {
        Ok(_) => println!("Documents inserted successfully."),
        Err(e) => eprintln!("Error inserting documents: {}", e),
    }

    println!("Done!");


    // Marketaux
    println!("\nSending GET reauest to Marketaux...");
    let _ = marketaux::example(&value_config).await;
    println!("Done!");

    // Alphavantage
    println!("\nSending GET reauest to Alpha Vantage...");
    let _ = alphavantage::example(&value_config).await;
    println!("Done!");

    Ok(())
}


#[tokio::main]
async fn _main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the configuration
    let config = config::ValueConfig::new().expect("Failed to load configuration");

    // Use the configuration values
    let api_alphavanatge = &config.api.alphavantage;
    let api_marketaux = &config.api.marketaux;
    let server_host = &config.server.host;
    let server_port = config.server.port;
    let logging_level = &config.logging.level;

    println!("API Alphavanatge: {}", api_alphavanatge);
    println!("API Marketaux: {}", api_marketaux);
    println!("Server Host: {}", server_host);
    println!("Server Port: {}", server_port);
    println!("Logging Level: {}", logging_level);

    // Your MongoDB client setup and other logic here...

    Ok(())
}

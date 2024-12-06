#[allow(dead_code)]

pub mod marketaux;
pub mod alphavantage;


#[tokio::main]
async fn main() {
    // Marketaux
    println!("\nSending GET reauest to Marketaux...");
    let _ = marketaux::example().await;
    println!("\nDone!\n");

    // Alphavantage
    println!("\nSending GET reauest to Alpha Vantage...");
    let _ = alphavantage::example().await;
    println!("\nDone!\n")
}

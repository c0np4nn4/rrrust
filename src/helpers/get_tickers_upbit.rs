use reqwest;
use serde::Deserialize;
use uuid::Uuid;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct MarketData {
    market: String,
    korean_name: String,
    english_name: String,
}

pub async fn get_all_tickers_upbit() -> Result<String, Box<dyn Error>> {
    let url = "https://api.upbit.com/v1/market/all";

    // Send the HTTP GET request
    let response = reqwest::get(url).await?.text().await?;

    // Parse the JSON response
    let markets: Vec<MarketData> = serde_json::from_str(&response)?;

    // Create subscription message
    let ticket = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let codes: Vec<String> = markets.iter().map(|market| market.market.clone()).collect();

    let subscription_message = serde_json::json!([
        { "ticket": ticket.to_string() },
        { "type": "ticker", "codes": codes }
    ]);

    // Return the subscription message as a string
    Ok(subscription_message.to_string())
}

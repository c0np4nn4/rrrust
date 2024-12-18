use chrono::{Local, TimeZone, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    self,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TickerData {
    code: String,
    trade_price: f64,
    trade_volume: f64,
    change: String,
    change_rate: f64,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    // Create an mpsc channel for requests and a task to handle them
    let (request_tx, mut request_rx) = mpsc::channel(32);

    tokio::spawn(async move {
        while let Some((url, response_tx)) = request_rx.recv().await {
            // Spawn a task to handle WebSocket connection and response
            tokio::spawn(async move {
                if let Err(err) = handle_websocket_request(url, response_tx).await {
                    eprintln!("Error handling WebSocket request: {}", err);
                }
            });
        }
    });

    // Example usage with Upbit WebSocket API
    let (response_tx, _response_rx) = oneshot::channel();
    request_tx
        .send(("wss://api.upbit.com/websocket/v1".to_string(), response_tx))
        .await
        .expect("Failed to send request");

    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await; // Run for 30 seconds
}

async fn handle_websocket_request(
    url: String,
    _response_tx: oneshot::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the URL and establish a WebSocket connection
    let (ws_stream, _) = connect_async(url.as_str()).await?;

    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    // Send a subscription message to the WebSocket server
    let subscription_message = r#"[
        {"ticket":"test"},
        {"type":"ticker","codes":["KRW-BTC"]}
    ]"#;

    write
        .send(Message::Text(subscription_message.to_string()))
        .await?;

    let mut recent_data: Vec<TickerData> = Vec::new();

    // Listen for messages and process them
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Binary(data)) => {
                match serde_json::from_slice::<TickerData>(&data) {
                    Ok(ticker) => {
                        // Maintain the last 5 data points
                        if recent_data.len() >= 8 {
                            recent_data.remove(0);
                        }
                        recent_data.push(ticker.clone());

                        // Clear screen for a fresh table display
                        print!("\x1B[2J\x1B[1;1H");

                        println!("|  Code      | Trade Price   | Trade Volume | Change | Change Rate | Local Timestamp       |");
                        println!("|------------|---------------|--------------|--------|-------------|-----------------------|");
                        for data in &recent_data {
                            let local_time = Local.timestamp_millis(data.timestamp as i64);
                            println!(
                                "| {:<10} | {:<13.2} | {:<12.4} | {:<6} | {:<11.4} | {:<21} |",
                                data.code,
                                data.trade_price,
                                data.trade_volume,
                                data.change,
                                data.change_rate,
                                local_time.format("%Y-%m-%d %H:%M:%S")
                            );
                        }
                    }
                    Err(err) => eprintln!("Failed to parse JSON to TickerData: {}", err),
                }
            }
            Ok(_) => {
                println!("Received non-binary message");
            }
            Err(err) => {
                eprintln!("Error reading message: {}", err);
                break;
            }
        }
    }

    Ok(())
}

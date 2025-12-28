use anyhow::{Result, Context};
use serde_json::Value;
use super::client::PolyClient;
use super::orderbook_parser::parse_price_levels;
impl PolyClient {
    pub async fn get_orderbook(&self, token_id: &str) -> Result<crate::types::orderbook::OrderBook> {
        let url = format!("https://clob.polymarket.com/book?token_id={}", token_id);
        println!("Fetching orderbook from: {}", url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch orderbook from CLOB API")?;
        println!("Orderbook API response status: {}", response.status());
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Orderbook API returned status: {}, Response: {}", status, error_text));
        }
        let json: Value = response.json().await
            .context("Failed to parse orderbook JSON")?;
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        if let Some(bids_array) = json.get("bids").and_then(|v| v.as_array()) {
            println!("Found {} bids in orderbook", bids_array.len());
            bids = parse_price_levels(bids_array);
        } else {
            println!("No 'bids' field found in orderbook response");
            let json_str = serde_json::to_string(&json).unwrap_or_default();
            println!("Orderbook JSON (first 200 chars): {}", &json_str.chars().take(200).collect::<String>());
        }
        if let Some(asks_array) = json.get("asks").and_then(|v| v.as_array()) {
            println!("Found {} asks in orderbook", asks_array.len());
            asks = parse_price_levels(asks_array);
        } else {
            println!("No 'asks' field found in orderbook response");
        }
        println!("Parsed {} bids, {} asks", bids.len(), asks.len());
        bids.sort_by(|a, b| {
            let a_price = a.price.parse::<f64>().unwrap_or(0.0);
            let b_price = b.price.parse::<f64>().unwrap_or(0.0);
            b_price.partial_cmp(&a_price).unwrap_or(std::cmp::Ordering::Equal)
        });
        asks.sort_by(|a, b| {
            let a_price = a.price.parse::<f64>().unwrap_or(0.0);
            let b_price = b.price.parse::<f64>().unwrap_or(0.0);
            a_price.partial_cmp(&b_price).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(crate::types::orderbook::OrderBook { bids, asks })
    }
}
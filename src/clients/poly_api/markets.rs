use anyhow::{Result, Context};
use crate::types::market::Market;
use serde_json::Value;
use super::client::PolyClient;
impl PolyClient {
    pub async fn get_active_btc_markets(&self) -> Result<Vec<Market>> {
        println!("--> Fetching active BTC markets from Polymarket API...");
        let url = "https://gamma-api.polymarket.com/markets?active=true&limit=100&tag_slug=bitcoin&order=volume24hr&ascending=false";
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch markets from Polymarket API")?;
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API returned status: {}, Response: {}", status, error_text));
        }
        let response_text = response.text().await
            .context("Failed to read API response")?;
        let json_value: Value = serde_json::from_str(&response_text)
            .map_err(|e| {
                eprintln!("Raw API response (first 500 chars): {}", &response_text.chars().take(500).collect::<String>());
                anyhow::anyhow!("Failed to parse JSON: {}. Response length: {}", e, response_text.len())
            })?;
        let markets_array: Vec<Value> = match json_value {
            Value::Array(arr) => arr,
            Value::Object(ref obj) => {
                if let Some(Value::Array(arr)) = obj.get("data") {
                    arr.clone()
                } else if let Some(Value::Array(arr)) = obj.get("markets") {
                    arr.clone()
                } else if let Some(Value::Array(arr)) = obj.get("results") {
                    arr.clone()
                } else {
                    return Err(anyhow::anyhow!("Expected array or object with 'data'/'markets'/'results' field. Got: {}", serde_json::to_string(obj).unwrap_or_default()));
                }
            }
            _ => return Err(anyhow::anyhow!("Expected array or object, got: {}", json_value)),
        };
        let json = markets_array;
        let mut markets = Vec::new();
        let mut parse_errors = 0;
        let mut expired_count = 0;
        let mut invalid_date_count = 0;
        let now = chrono::Utc::now();
        for (idx, market_data) in json.iter().enumerate() {
            let question = market_data.get("question")
                .or_else(|| market_data.get("title"))
                .and_then(|q| q.as_str())
                .unwrap_or("");
            if question.is_empty() || question.to_lowercase().contains("btc") || question.to_lowercase().contains("bitcoin") {
                match Self::parse_market(market_data) {
                    Ok(market) => {
                        if market.end_date_iso.is_empty() {
                            invalid_date_count += 1;
                            continue;
                        }
                        match chrono::DateTime::parse_from_rfc3339(&market.end_date_iso) {
                            Ok(expiry_time) => {
                                let expiry_utc = expiry_time.with_timezone(&chrono::Utc);
                                if expiry_utc > now {
                                    markets.push(market);
                                } else {
                                    expired_count += 1;
                                    if expired_count == 1 {
                                        let days_expired = (now - expiry_utc).num_days();
                                        println!("Filtered expired market: {} (expired {} days ago)",
                                            market.question, days_expired);
                                    }
                                }
                            }
                            Err(_) => {
                                invalid_date_count += 1;
                                if invalid_date_count == 1 {
                                    println!("Invalid expiry date format: '{}' for market: {}",
                                        market.end_date_iso, market.question);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        parse_errors += 1;
                        if parse_errors <= 3 {
                            eprintln!("Failed to parse market #{}: {}", idx, e);
                            eprintln!("Market data: {}", serde_json::to_string(market_data).unwrap_or_default().chars().take(200).collect::<String>());
                        }
                    }
                }
            }
        }
        if expired_count > 0 {
            println!("Filtered out {} expired market(s)", expired_count);
        }
        if invalid_date_count > 0 {
            println!("Skipped {} market(s) with invalid expiry dates", invalid_date_count);
        }
        if parse_errors > 0 {
            eprintln!("Total parse errors: {} out of {} markets", parse_errors, json.len());
        }
        println!("Found {} BTC markets", markets.len());
        Ok(markets)
    }
    pub fn parse_market(data: &Value) -> Result<Market> {
        super::market_parser::parse_market(data)
    }
    pub async fn get_market_by_condition(&self, condition_id: &str) -> Result<Market> {
        super::market_fetcher::get_market_by_condition(self, condition_id).await
    }
}
use anyhow::{Result, Context};
use serde_json::Value;
use super::client::PolyClient;
pub async fn get_market_by_condition(client: &PolyClient, condition_id: &str) -> Result<crate::types::market::Market> {
        let url = format!("https://gamma-api.polymarket.com/markets?active=true&conditionId={}", condition_id);
        let response = client.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch markets")?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("API returned status: {}", response.status()));
        }
        let response_text = response.text().await
            .context("Failed to read API response")?;
        let json_value: Value = serde_json::from_str(&response_text)
            .context("Failed to parse markets JSON")?;
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
                    return Err(anyhow::anyhow!("Market not found for condition ID: {}", condition_id));
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid response format for condition ID: {}", condition_id)),
        };
        for market_data in markets_array {
            let cond_id = market_data.get("conditionId")
                .or_else(|| market_data.get("condition_id"))
                .or_else(|| market_data.get("id"))
                .and_then(|v| v.as_str());
            if let Some(cond_id) = cond_id {
                if cond_id == condition_id {
                    return super::market_parser::parse_market(&market_data);
                }
            }
        }
        Err(anyhow::anyhow!("Market not found for condition ID: {}", condition_id))
}
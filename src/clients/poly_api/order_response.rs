use anyhow::{Result, Context};
use serde_json::Value;

pub fn handle_order_response(status: reqwest::StatusCode, response_text: String) -> Result<String> {
    println!("API Response Status: {}", status);
    
    let response_preview = if response_text.len() > 500 {
        format!("{}... (truncated)", &response_text[..500])
    } else {
        response_text.clone()
    };
    println!("API Response Body: {}", response_preview);
    
    // Attempt to extract JSON from HTML-wrapped responses (common with FlareSolverr)
    let mut json_response_text = response_text.clone();
    if json_response_text.contains("<pre>") && json_response_text.contains("</pre>") {
        if let Some(start) = json_response_text.find("<pre>") {
            if let Some(end) = json_response_text.find("</pre>") {
                json_response_text = json_response_text[start + "<pre>".len()..end].trim().to_string();
            }
        }
    }

    // Even if status is success, check for API errors in the body
    if let Ok(error_json) = serde_json::from_str::<Value>(&json_response_text) {
        if let Some(error_msg) = error_json.get("error").and_then(|v| v.as_str()) {
            return Err(anyhow::anyhow!(
                "API Error: {}\nThis usually means:\n   - API key is invalid or expired\n   - API credentials don't match your wallet\n   - API key doesn't have trading permissions\nResponse: {}",
                error_msg, response_preview
            ));
        }
    }

    if !status.is_success() {
        if status == 403 {
            return Err(anyhow::anyhow!(
                "Cloudflare blocked request (403 Forbidden).\n\
                 Solution: Ensure FlareSolverr is running and has a valid session.\n\
                 Status: {}, Response: {}",
                status, response_preview
            ));
        }
        
        if status == 400 {
            if let Ok(error_json) = serde_json::from_str::<Value>(&json_response_text) {
                if let Some(error_msg) = error_json.get("error").and_then(|v| v.as_str()) {
                    return Err(anyhow::anyhow!(
                        "Order placement failed (400 Bad Request): {}\nFull response: {}",
                        error_msg, response_preview
                    ));
                }
            }
        }
        
        return Err(anyhow::anyhow!(
            "Order placement failed. Status: {}, Response: {}",
            status, response_preview
        ));
    }
    
    let trimmed = json_response_text.trim_start();
    let is_json = trimmed.starts_with('{') || trimmed.starts_with('[');
    if !is_json {
        let looks_like_html = trimmed.to_lowercase().contains("<html") || 
                              trimmed.to_lowercase().contains("cloudflare") ||
                              trimmed.to_lowercase().contains("challenge");
        
        if looks_like_html {
            return Err(anyhow::anyhow!(
                "Cloudflare challenge detected in response (HTML instead of JSON).\n\
                 FlareSolverr may need a fresh session or longer timeout.\n\
                 Response preview: {}", response_preview
            ));
        } else {
            return Err(anyhow::anyhow!(
                "Non-JSON response from Polymarket API.\n\
                 This may indicate a Cloudflare block or API error.\n\
                 Response preview: {}", response_preview
            ));
        }
    }
    
    let response_json: Value = serde_json::from_str(&json_response_text)
        .context("Failed to parse API response as JSON")?;
    
    let order_id = response_json
        .get("orderId")
        .or_else(|| response_json.get("id"))
        .or_else(|| response_json.get("hash"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!("Order ID not found in API response: {}", json_response_text)
        })?;
    
    println!("Order placed successfully! Order ID: {}", order_id);
    Ok(order_id.to_string())
}

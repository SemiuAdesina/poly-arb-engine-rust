use anyhow::{Result, Context};
use serde_json::Value;
use super::super::client::PolyClient;

pub async fn parse_flaresolverr_response(_client: &PolyClient, response: reqwest::Response) -> Result<(reqwest::StatusCode, String)> {
    let response_status = response.status();
    let response_text = response.text().await
        .context("Failed to read FlareSolverr response")?;
    
    println!("FlareSolverr response status: {}", response_status);
    println!("FlareSolverr response preview: {}", response_text.chars().take(500).collect::<String>());
    
    let flaresolverr_response: Value = match serde_json::from_str(&response_text) {
        Ok(v) => v,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "FlareSolverr returned non-JSON response (likely Cloudflare HTML). \
Status: {}, Error: {}, Response preview: {}",
                response_status,
                e,
                response_text.chars().take(500).collect::<String>()
            ));
        }
    };
    
    if flaresolverr_response.get("status") != Some(&Value::String("ok".to_string())) {
        let message = flaresolverr_response
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown FlareSolverr error");
        return Err(anyhow::anyhow!("FlareSolverr error: {}", message));
    }
    
    let solution = flaresolverr_response.get("solution")
        .ok_or_else(|| {
            eprintln!("FlareSolverr response missing 'solution' field");
            eprintln!("Full FlareSolverr response: {}", serde_json::to_string(&flaresolverr_response).unwrap_or_default());
            anyhow::anyhow!("No solution in FlareSolverr response. Check FlareSolverr logs.")
        })?;
    
    let status_code = solution.get("status")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u16;
    
    let response_body = solution.get("body")
        .or_else(|| solution.get("response"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    if response_body.is_empty() {
        eprintln!("FlareSolverr returned empty response body");
        eprintln!("Solution object: {}", serde_json::to_string(solution).unwrap_or_default());
    } else if response_body.len() < 10 {
        eprintln!("FlareSolverr returned suspiciously short response: {}", response_body);
    }
    
    let status = reqwest::StatusCode::from_u16(status_code)
        .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
    
    Ok((status, response_body))
}

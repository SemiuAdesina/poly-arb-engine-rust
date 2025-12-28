use anyhow::{Result, Context, anyhow};
use serde_json::Value;
use std::process::Command;
use super::super::client::PolyClient;
pub async fn handle_curl_fallback(_client: &PolyClient, url: &str, payload: &Value) -> Result<(reqwest::StatusCode, String)> {
    log::debug!("Reqwest failed to connect to FlareSolverr. Trying curl fallback...");
    let payload_str = serde_json::to_string(payload)
        .context("serializing payload for curl")?;
    let output = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(&payload_str)
        .arg("--max-time")
        .arg("120")
        .arg("--silent")
        .arg("--show-error")
        .arg(url)
        .output()
        .context("executing curl fallback for FlareSolverr")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("curl fallback failed: {}", stderr));
    }
    let flaresolverr_response: Value = serde_json::from_slice(&output.stdout)
        .context("parsing curl JSON response")?;
    if flaresolverr_response.get("status") != Some(&Value::String("ok".to_string())) {
        let message = flaresolverr_response
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown FlareSolverr error");
        return Err(anyhow!("FlareSolverr error: {}", message));
    }
    let solution = flaresolverr_response.get("solution")
        .ok_or_else(|| anyhow!("No solution in FlareSolverr response"))?;
    let status_code = solution.get("status")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u16;
    let response_body = solution.get("body")
        .or_else(|| solution.get("response"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let status = reqwest::StatusCode::from_u16(status_code)
        .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
    Ok((status, response_body))
}
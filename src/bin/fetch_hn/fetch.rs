use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use std::process::Command;
pub async fn fetch_hn_html(bot_url: &str, session: &str, target_url: &str) -> Result<String> {
    let payload = serde_json::json!({
        "cmd": "request.get",
        "session": session,
        "url": target_url,
        "render": true,
        "maxTimeout": 60_000
    });
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("building HTTP client")?;
    match client
        .post(bot_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err(anyhow!("bot returned HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.context("parsing bot JSON")?;
            let status = body
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if status != "ok" {
                return Err(anyhow!("bot response not ok: {body:?}"));
            }
            return body
                .get("solution")
                .and_then(|v| v.get("response"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("missing solution.response in bot payload"));
        }
        Err(e) => {
            log::debug!("Reqwest failed: {}. Falling back to curl...", e);
            return fetch_hn_html_curl(bot_url, session, target_url);
        }
    }
}
fn fetch_hn_html_curl(bot_url: &str, session: &str, target_url: &str) -> Result<String> {
    let payload = serde_json::json!({
        "cmd": "request.get",
        "session": session,
        "url": target_url,
        "render": true,
        "maxTimeout": 60_000
    });
    let payload_str = serde_json::to_string(&payload)
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
        .arg(bot_url)
        .output()
        .context("executing curl command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("curl failed: {}", stderr));
    }
    let body: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("parsing curl JSON response")?;
    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if status != "ok" {
        return Err(anyhow!("bot response not ok: {body:?}"));
    }
    body.get("solution")
        .and_then(|v| v.get("response"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("missing solution.response in bot payload"))
}
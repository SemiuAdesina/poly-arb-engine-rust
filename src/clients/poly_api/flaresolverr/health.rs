use anyhow::Result;
use std::time::Duration;
use std::process::Command;
use super::super::client::PolyClient;

pub async fn ensure_flaresolverr(client: &PolyClient) -> Result<&String> {
    let flaresolverr_url = client.flaresolverr_url.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "FlareSolverr not configured. Direct connection will be blocked by Cloudflare.\n\
Set FLARESOLVERR_URL=http://127.0.0.1:8191 (or http://127.0.0.1:8191/v1) in .env\n\
Then run: docker run -d -p 8191:8191 ghcr.io/flaresolverr/flaresolverr:latest"
        )
    })?;
    
    let base_url = flaresolverr_url.trim_end_matches('/');
    let health_url = if base_url.ends_with("/v1") {
        base_url.to_string()
    } else {
        format!("{}/v1", base_url)
    };
    
    let resp = client.client.get(&health_url).timeout(Duration::from_secs(10)).send().await;
    
    match resp {
        Ok(r) if r.status().is_success() => Ok(flaresolverr_url),
        Ok(r) => Err(anyhow::anyhow!(
            "FlareSolverr reachable but returned non-200 (status {}): {:?}\n\
Ensure FlareSolverr is running and reachable at {}",
            r.status(),
            r.text().await.unwrap_or_default(),
            flaresolverr_url
        )),
        Err(e) => {
            log::debug!("Reqwest failed to connect to FlareSolverr: {}. Trying curl fallback...", e);
            if let Ok(output) = Command::new("curl")
                .arg("--max-time")
                .arg("10")
                .arg("--silent")
                .arg("--show-error")
                .arg(&health_url)
                .output()
            {
                if output.status.success() {
                    log::debug!("Curl fallback succeeded for FlareSolverr health check");
                    return Ok(flaresolverr_url);
                }
            }
            Err(anyhow::anyhow!(
                "Failed to reach FlareSolverr at {}: {}\n\
Start it with: docker run -d -p 8191:8191 ghcr.io/flaresolverr/flaresolverr:latest\n\
On macOS, try: export FLARESOLVERR_URL=http://127.0.0.1:8191 (or http://127.0.0.1:8191/v1)",
                flaresolverr_url,
                e
            ))
        },
    }
}

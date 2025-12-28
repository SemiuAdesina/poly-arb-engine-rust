pub mod health;
pub mod headers;
pub mod curl_fallback;
pub mod response_parser;
pub mod post;
pub mod delete;
pub trait FlareSolverrClient {
    async fn ensure_flaresolverr(&self) -> anyhow::Result<&String>;
    async fn send_post_via_flaresolverr(
        &self,
        flaresolverr_url: &str,
        target_url: &str,
        body: &str,
        request_signature: &str,
        timestamp: i64,
        address: &str,
    ) -> anyhow::Result<(reqwest::StatusCode, String)>;
    async fn send_delete_via_flaresolverr(
        &self,
        flaresolverr_url: &str,
        target_url: &str,
        request_signature: &str,
        timestamp: i64,
        address: &str,
    ) -> anyhow::Result<(reqwest::StatusCode, String)>;
}
use super::client::PolyClient;
impl FlareSolverrClient for PolyClient {
    async fn ensure_flaresolverr(&self) -> anyhow::Result<&String> {
        health::ensure_flaresolverr(self).await
    }
    async fn send_post_via_flaresolverr(
        &self,
        flaresolverr_url: &str,
        target_url: &str,
        body: &str,
        request_signature: &str,
        timestamp: i64,
        address: &str,
    ) -> anyhow::Result<(reqwest::StatusCode, String)> {
        post::send_post_via_flaresolverr(
            self, flaresolverr_url, target_url, body, request_signature, timestamp, address
        ).await
    }
    async fn send_delete_via_flaresolverr(
        &self,
        flaresolverr_url: &str,
        target_url: &str,
        request_signature: &str,
        timestamp: i64,
        address: &str,
    ) -> anyhow::Result<(reqwest::StatusCode, String)> {
        delete::send_delete_via_flaresolverr(
            self, flaresolverr_url, target_url, request_signature, timestamp, address
        ).await
    }
}
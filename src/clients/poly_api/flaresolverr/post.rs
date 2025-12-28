use anyhow::Result;
use super::super::client::PolyClient;
use super::headers;
use super::curl_fallback;
use super::response_parser;
pub async fn send_post_via_flaresolverr(
        client: &PolyClient,
        flaresolverr_url: &str,
        target_url: &str,
        body: &str,
        request_signature: &str,
        timestamp: i64,
        address: &str,
    ) -> Result<(reqwest::StatusCode, String)> {
        let headers_map = headers::build_flaresolverr_headers(client, request_signature, timestamp, address);
        let mut flaresolverr_payload = serde_json::json!({
            "cmd": "request.post",
            "url": target_url,
            "postData": body,
            "headers": headers_map,
            "maxTimeout": 60000
        });
        if let Ok(session) = std::env::var("FLARESOLVERR_SESSION") {
            flaresolverr_payload["session"] = serde_json::Value::String(session);
        }
        if let Some(ref proxy) = client.flaresolverr_proxy {
            flaresolverr_payload["proxy"] = serde_json::Value::String(proxy.clone());
            println!("Using proxy: {}", proxy);
        }
        let flaresolverr_api_url = headers::get_flaresolverr_api_url(client, flaresolverr_url);
        println!("FlareSolverr URL: {}", flaresolverr_api_url);
        let response = match client.client
            .post(&flaresolverr_api_url)
            .json(&flaresolverr_payload)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_e) => return curl_fallback::handle_curl_fallback(client, &flaresolverr_api_url, &flaresolverr_payload).await,
        };
        response_parser::parse_flaresolverr_response(client, response).await
}
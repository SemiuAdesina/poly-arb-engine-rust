use serde_json::{Map, Value};
use super::super::client::PolyClient;
pub fn build_flaresolverr_headers(client: &PolyClient, request_signature: &str, timestamp: i64, address: &str) -> Map<String, Value> {
    let chrome_user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let mut headers_map = Map::new();
    headers_map.insert("Content-Type".to_string(), Value::String("application/json".to_string()));
    headers_map.insert("User-Agent".to_string(), Value::String(chrome_user_agent.to_string()));
    headers_map.insert("x-api-key".to_string(), Value::String(client.api_key.clone()));
    headers_map.insert("POLY_API_KEY".to_string(), Value::String(client.api_key.clone()));
    headers_map.insert("POLY_SIGNATURE".to_string(), Value::String(request_signature.to_string()));
    headers_map.insert("POLY_TIMESTAMP".to_string(), Value::String(timestamp.to_string()));
    headers_map.insert("POLY_ADDRESS".to_string(), Value::String(address.to_string()));
    headers_map.insert("POLY_PASSPHRASE".to_string(), Value::String(client.api_passphrase.clone()));
    headers_map.insert("Accept".to_string(), Value::String("application/json".to_string()));
    headers_map.insert("Accept-Language".to_string(), Value::String("en-US,en;q=0.9".to_string()));
    headers_map.insert("Accept-Encoding".to_string(), Value::String("identity".to_string()));
    headers_map.insert("Origin".to_string(), Value::String("https://polymarket.com".to_string()));
    headers_map.insert("Referer".to_string(), Value::String("https://polymarket.com/".to_string()));
    headers_map.insert("Sec-Fetch-Dest".to_string(), Value::String("empty".to_string()));
    headers_map.insert("Sec-Fetch-Mode".to_string(), Value::String("cors".to_string()));
    headers_map.insert("Sec-Fetch-Site".to_string(), Value::String("same-site".to_string()));
    if let Some(ref cookie) = client.cloudflare_cookie {
        headers_map.insert("Cookie".to_string(), Value::String(cookie.clone()));
    }
    headers_map
}
pub fn get_flaresolverr_api_url(_client: &PolyClient, flaresolverr_url: &str) -> String {
    let base_url = flaresolverr_url.trim_end_matches('/');
    if base_url.ends_with("/v1") {
        base_url.to_string()
    } else {
        format!("{}/v1", base_url)
    }
}
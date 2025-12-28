use anyhow::{Result, Context};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;
type HmacSha256 = Hmac<Sha256>;
pub fn sign_request(api_secret: &str, timestamp: i64, method: &str, path: &str, body: &str) -> Result<String> {
    let secret_bytes = base64::engine::general_purpose::URL_SAFE.decode(api_secret)
        .context("Failed to base64 decode API secret (should be URL-safe base64)")?;
    let message = format!("{}{}{}{}", timestamp, method, path, body);
    let mut mac = HmacSha256::new_from_slice(&secret_bytes)
        .context("Failed to create HMAC from decoded API secret")?;
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let signature_bytes = result.into_bytes();
    let signature = base64::engine::general_purpose::URL_SAFE.encode(&signature_bytes);
    Ok(signature)
}
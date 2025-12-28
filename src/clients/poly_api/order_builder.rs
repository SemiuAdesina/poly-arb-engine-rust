use anyhow::{Result, Context};
use super::client::PolyClient;
use super::types::{OrderSide, get_next_nonce};
pub async fn build_order_payload(
    client: &PolyClient,
    token_id: &str,
    size: f64,
    price: f64,
    order_side: OrderSide,
    signer: &crate::execution::signer::OrderSigner,
) -> Result<(String, String, i64, String)> {
    let size_scaled = (size * 1_000_000_000.0).round() as u64;
    let size_wei = (size_scaled as u128) * 1_000_000_000;
    let price_usdc_scaled = (price * 1_000_000.0).round() as u128;
    let usdc_amount = (price_usdc_scaled * size_wei) / 1_000_000_000_000_000_000u128;
    let expiration_timestamp = 0i64;
    let fee_rate = 0u64;
    let salt = chrono::Utc::now().timestamp_millis() as u64;
    let nonce = get_next_nonce();
    let (maker_amount_for_signing, taker_amount_for_signing) = if order_side.is_buy() {
        (usdc_amount, size_wei)
    } else {
        (size_wei, usdc_amount)
    };
    println!("Signing order with EIP-712...");
    let signature = signer.sign_order_with_amounts(
        token_id,
        maker_amount_for_signing,
        taker_amount_for_signing,
        order_side.is_buy(),
        expiration_timestamp,
        fee_rate,
        Some(salt),
        Some(nonce),
    ).context("Failed to sign order with EIP-712")?;
    println!("Order signed successfully");
    let signing_address = signer.signing_address();
    let maker_address = format!("{:#x}", signing_address).to_lowercase();
    let signer_address = maker_address.clone();
    let (maker_amount, taker_amount) = if order_side.is_buy() {
        (usdc_amount.to_string(), size_wei.to_string())
    } else {
        (size_wei.to_string(), usdc_amount.to_string())
    };
    let order_obj = serde_json::json!({
        "salt": salt,
        "maker": maker_address,
        "signer": signer_address,
        "taker": "0x0000000000000000000000000000000000000000",
        "tokenId": token_id,
        "makerAmount": maker_amount,
        "takerAmount": taker_amount,
        "expiration": expiration_timestamp,
        "nonce": nonce,
        "feeRateBps": fee_rate,
        "side": if order_side.is_buy() { 0 } else { 1 },
        "signature": signature,
    });
    let order_payload = serde_json::json!({
        "order": order_obj,
        "owner": client.api_key.clone(),
        "orderType": "GTC",
    });
    println!("Order payload constructed");
    let timestamp = chrono::Utc::now().timestamp();
    let order_body = serde_json::to_string(&order_payload)
        .context("Failed to serialize order payload")?;
    println!("Order payload JSON: {}", order_body);
    let path = "/order";
    let request_signature = super::auth::sign_request(&client.api_secret, timestamp, "POST", path, &order_body)
        .context("Failed to sign HTTP request")?;
    println!("HTTP request signed with HMAC");
    let address = format!("{:#x}", signer.address()).to_lowercase();
    Ok((order_body, request_signature, timestamp, address))
}
use anyhow::{Result, Context};
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::execution::signer::OrderSigner;
pub async fn execute_manual_buy(
    settings: &Settings,
    market_id: &str,
    usdc_amount: f64,
    yes_price: f64,
) -> Result<String> {
    println!("--> EXECUTING MANUAL BUY ORDER...");
    println!("Market: {}", market_id);
    println!("Amount: ${:.2} USDC", usdc_amount);
    println!("YES price: ${:.4}", yes_price);
    let shares = usdc_amount / yes_price;
    println!("Shares: {:.4}", shares);
    let signer = OrderSigner::new(settings)
        .context("Failed to initialize order signer")?;
    let api_client = PolyClient::new(settings);
    println!("Placing order...");
    let order_id = api_client.place_order(market_id, "YES", shares, yes_price, &signer).await
        .context("Failed to place order")?;
    println!("Manual buy order executed!");
    println!("Order ID: {}", order_id);
    Ok(order_id)
}
pub async fn execute_manual_sell(
    settings: &Settings,
    market_id: &str,
    shares: f64,
    sell_price: f64,
) -> Result<String> {
    println!("--> EXECUTING MANUAL SELL ORDER...");
    println!("Market: {}", market_id);
    println!("Shares: {:.4}", shares);
    println!("Sell price: ${:.4}", sell_price);
    let signer = OrderSigner::new(settings)
        .context("Failed to initialize order signer")?;
    let api_client = PolyClient::new(settings);
    println!("Placing sell order...");
    let order_id = api_client.place_order(market_id, "YES", shares, sell_price, &signer).await
        .context("Failed to place sell order")?;
    println!("Manual sell order executed!");
    println!("Order ID: {}", order_id);
    Ok(order_id)
}
use anyhow::{Result, Context};
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::types::market::Market;
use crate::execution::signer::OrderSigner;
use tokio::sync::mpsc::UnboundedSender;
pub async fn place_no_order(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    settings: &Settings,
    market: &Market,
    no_size: f64,
    no_limit: f64,
    profit_pct: f64,
    expiry_utc: chrono::DateTime<chrono::Utc>,
    alert_tx: &UnboundedSender<String>,
) -> Result<usize> {
    let signer = OrderSigner::new(settings)
        .context("Failed to initialize order signer")?;
    match poly_client.place_order_with_token_id(
        &market.condition_id,
        market.token_id_no.clone(),
        crate::clients::poly_api::OutcomeSide::No,
        no_size,
        no_limit,
        crate::clients::poly_api::OrderSide::Sell,
        &signer,
    ).await {
        Ok(order_id) => {
            println!("NO limit order placed: {:.1} shares @ ${:.4} (value: ${:.2})",
                     no_size, no_limit, no_size * no_limit);
            let pending_order = crate::types::pending_order::PendingOrder::new(
                market.condition_id.clone(),
                market.question.clone(),
                market.token_id_no.clone(),
                "NO".to_string(),
                order_id.clone(),
                no_limit,
                no_size,
                expiry_utc,
            );
            pending_order_manager.add_order(pending_order).await;
            let msg = format!("Fishing Orders Placed\nMarket: {}\nYES: ${:.4} | NO: ${:.4}\nExpected profit: {:.2}%",
                market.question, no_limit, no_limit, profit_pct
            );
            let _ = alert_tx.send(msg);
            Ok(1)
        }
        Err(e) => {
            eprintln!("Failed to place NO order: {}", e);
            Ok(0)
        }
    }
}
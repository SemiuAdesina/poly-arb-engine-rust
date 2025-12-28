use anyhow::{Result, Context};
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::types::market::Market;
use crate::execution::signer::OrderSigner;
pub async fn place_yes_order(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    settings: &Settings,
    market: &Market,
    yes_size: f64,
    yes_limit: f64,
    expiry_utc: chrono::DateTime<chrono::Utc>,
) -> Result<usize> {
    let signer = OrderSigner::new(settings)
        .context("Failed to initialize order signer")?;
    match poly_client.place_order_with_token_id(
        &market.condition_id,
        market.token_id_yes.clone(),
        crate::clients::poly_api::OutcomeSide::Yes,
        yes_size,
        yes_limit,
        crate::clients::poly_api::OrderSide::Buy,
        &signer,
    ).await {
        Ok(order_id) => {
            println!("YES limit order placed: {:.1} shares @ ${:.4} (value: ${:.2})",
                     yes_size, yes_limit, yes_size * yes_limit);
            let pending_order = crate::types::pending_order::PendingOrder::new(
                market.condition_id.clone(),
                market.question.clone(),
                market.token_id_yes.clone(),
                "YES".to_string(),
                order_id.clone(),
                yes_limit,
                yes_size,
                expiry_utc,
            );
            pending_order_manager.add_order(pending_order).await;
            Ok(1)
        }
        Err(e) => {
            eprintln!("Failed to place YES order: {}", e);
            Ok(0)
        }
    }
}
use anyhow::{Result, Context};
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::execution::signer::OrderSigner;
pub async fn cancel_orders_near_expiry(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    settings: &Settings,
) -> Result<()> {
    let orders_to_cancel = pending_order_manager.get_orders_to_cancel().await;
    if orders_to_cancel.is_empty() {
        return Ok(());
    }
    println!("Cancelling {} order(s) too close to expiry...", orders_to_cancel.len());
    let signer = OrderSigner::new(settings)
        .context("Failed to initialize order signer")?;
    for order in orders_to_cancel {
        match poly_client.cancel_order(&order.order_id, &signer).await {
            Ok(_) => {
                println!("Cancelled order: {} ({})", &order.order_id[..16], order.side);
                pending_order_manager.remove_order(&order.order_id).await;
            }
            Err(e) => {
                eprintln!("Failed to cancel order {}: {}", &order.order_id[..16], e);
            }
        }
    }
    Ok(())
}
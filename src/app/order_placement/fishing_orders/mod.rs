pub mod market_loop;
pub mod order_handlers;
use anyhow::Result;
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::engine::positions::PositionManager;
use crate::types::market::Market;
use tokio::sync::mpsc::UnboundedSender;
pub async fn place_fishing_orders(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    position_manager: &PositionManager,
    settings: &Settings,
    markets: Vec<Market>,
    alert_tx: &UnboundedSender<String>,
) -> Result<usize> {
    let orders_placed = market_loop::process_markets(
        poly_client, pending_order_manager, position_manager, settings, markets, alert_tx
    ).await?;
    if orders_placed == 0 {
        println!("No fishing opportunities found in this cycle");
        if settings.test_mode {
            println!("TEST_MODE: Markets were checked but no orders placed (check 'Market analysis' logs above)");
        }
    } else {
        println!("Placed {} order(s) in this cycle", orders_placed);
    }
    Ok(orders_placed)
}
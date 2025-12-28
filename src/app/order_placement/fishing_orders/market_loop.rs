use anyhow::Result;
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::engine::positions::PositionManager;
use crate::types::market::Market;
use tokio::sync::mpsc::UnboundedSender;
use super::order_handlers;
pub async fn process_markets(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    position_manager: &PositionManager,
    settings: &Settings,
    markets: Vec<Market>,
    alert_tx: &UnboundedSender<String>,
) -> Result<usize> {
    let total_markets = markets.len();
    let valid_markets = crate::engine::discovery::find_all_valid_markets(markets);
    println!("Filtered markets: {} valid out of {} total (looking for 1-72 hour expiry)",
             valid_markets.len(), total_markets);
    if valid_markets.is_empty() {
        println!("No markets in 6-24 hour expiry window");
        println!("In TEST_MODE, this is okay - markets might expire in different timeframes");
        return Ok(0);
    }
    let mut orders_placed = 0;
    for market in valid_markets.iter().take(5) {
        if pending_order_manager.has_complete_set(&market.condition_id).await {
            continue;
        }
        let positions = position_manager.get_positions().await;
        if positions.iter().any(|p| p.condition_id == market.condition_id) {
            continue;
        }
        println!("Checking fishing opportunity: {}", market.question);
        let yes_ob_result = poly_client.get_orderbook(&market.token_id_yes).await;
        let no_ob_result = poly_client.get_orderbook(&market.token_id_no).await;
        match (yes_ob_result, no_ob_result) {
            (Ok(yes_ob), Ok(no_ob)) => {
                let yes_bid = yes_ob.bids.first()
                    .and_then(|b| b.price.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let no_bid = no_ob.bids.first()
                    .and_then(|b| b.price.parse::<f64>().ok())
                    .unwrap_or(0.0);
                println!("Market analysis: {} | YES bid=${:.4} | NO bid=${:.4}",
                         market.question, yes_bid, no_bid);
                if settings.test_mode {
                    orders_placed += order_handlers::handle_test_mode_order(
                        poly_client, pending_order_manager, settings, market,
                        yes_bid, no_bid, alert_tx
                    ).await?;
                } else if yes_bid > 0.0 && no_bid > 0.0 {
                    orders_placed += order_handlers::handle_production_order(
                        poly_client, pending_order_manager, settings, market,
                        yes_bid, no_bid, alert_tx
                    ).await?;
                } else {
                    println!("No bids available: YES bid=${:.4}, NO bid=${:.4}", yes_bid, no_bid);
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("Failed to fetch orderbook for {}: {}", market.question, e);
            }
        }
    }
    Ok(orders_placed)
}
use anyhow::Result;
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::types::market::Market;
use tokio::sync::mpsc::UnboundedSender;
use super::super::order_execution;
pub async fn handle_test_mode_order(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    settings: &Settings,
    market: &Market,
    yes_bid: f64,
    no_bid: f64,
    alert_tx: &UnboundedSender<String>,
) -> Result<usize> {
    println!("TEST MODE: Bypassing all checks! Force placing orders...");
    let yes_bid_use = if yes_bid > 0.0 { yes_bid } else { 0.50 };
    let no_bid_use = if no_bid > 0.0 { no_bid } else { 0.50 };
    println!("Using prices: YES=${:.4}, NO=${:.4} (real bids: YES=${:.4}, NO=${:.4})",
             yes_bid_use, no_bid_use, yes_bid, no_bid);
    let (should_fish, yes_limit, no_limit, profit_pct) =
        crate::engine::fishing::should_place_fishing_orders(yes_bid_use, no_bid_use, 0.0, true);
    if !should_fish {
        eprintln!("TEST MODE: should_place_fishing_orders returned false unexpectedly");
        return Ok(0);
    }
    println!("TEST MODE: Force placing orders! YES limit=${:.4}, NO limit=${:.4}",
             yes_limit, no_limit);
    order_execution::place_orders_for_market(
        poly_client, pending_order_manager, settings, market,
        yes_limit, no_limit, profit_pct, alert_tx
    ).await
}
pub async fn handle_production_order(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    settings: &Settings,
    market: &Market,
    yes_bid: f64,
    no_bid: f64,
    alert_tx: &UnboundedSender<String>,
) -> Result<usize> {
    let profit_thresholds = [0.8, 1.5, 2.0, 2.5];
    let random_index = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize) % profit_thresholds.len();
    let min_profit_pct = profit_thresholds[random_index];
    println!("Using profit threshold: {:.1}% (varied from [0.8, 1.5, 2.0, 2.5])", min_profit_pct);
    let (should_fish, yes_limit, no_limit, profit_pct) =
        crate::engine::fishing::should_place_fishing_orders(yes_bid, no_bid, min_profit_pct, false);
    if !should_fish {
        println!("Opportunity rejected: Profit {:.2}% < minimum {:.2}%",
                 profit_pct, min_profit_pct);
        return Ok(0);
    }
    println!("Fishing opportunity found! Expected profit: {:.2}%", profit_pct);
    order_execution::place_orders_for_market(
        poly_client, pending_order_manager, settings, market,
        yes_limit, no_limit, profit_pct, alert_tx
    ).await
}
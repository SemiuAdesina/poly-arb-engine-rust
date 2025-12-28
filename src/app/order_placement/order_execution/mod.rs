pub mod yes_order;
pub mod no_order;
use anyhow::{Result, Context};
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::types::market::Market;
use tokio::sync::mpsc::UnboundedSender;
pub async fn place_orders_for_market(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    settings: &Settings,
    market: &Market,
    yes_limit: f64,
    no_limit: f64,
    profit_pct: f64,
    alert_tx: &UnboundedSender<String>,
) -> Result<usize> {
    let min_order_value = 5.0;
    let yes_size = crate::engine::fishing::calculate_minimum_order_size(yes_limit, min_order_value);
    let no_size = crate::engine::fishing::calculate_minimum_order_size(no_limit, min_order_value);
    println!("Order sizes: YES={:.1} shares, NO={:.1} shares", yes_size, no_size);
    let total_order_value = (yes_size * yes_limit) + (no_size * no_limit);
    println!("Order values: YES=${:.2}, NO=${:.2}, Total=${:.2}",
             yes_size * yes_limit, no_size * no_limit, total_order_value);
    use crate::app::order_placement::safety_checks;
    if !safety_checks::check_safety_limits(settings, pending_order_manager, total_order_value).await? {
        return Ok(0);
    }
    let expiry_utc = chrono::DateTime::parse_from_rfc3339(&market.end_date_iso)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .context("Failed to parse expiry time")?;
    let existing_orders = pending_order_manager.get_orders_for_market(&market.condition_id).await;
    let has_yes = existing_orders.iter().any(|o| o.side == "YES");
    let has_no = existing_orders.iter().any(|o| o.side == "NO");
    let mut orders_placed = 0;
    if !has_yes {
        orders_placed += yes_order::place_yes_order(
            poly_client, pending_order_manager, settings, market, yes_size, yes_limit, expiry_utc
        ).await?;
    }
    if !has_no {
        orders_placed += no_order::place_no_order(
            poly_client, pending_order_manager, settings, market, no_size, no_limit, profit_pct,
            expiry_utc, alert_tx
        ).await?;
    }
    Ok(orders_placed)
}
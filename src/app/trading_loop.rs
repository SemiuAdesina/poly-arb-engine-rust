use anyhow::Result;
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::engine::positions::PositionManager;
use tokio::sync::mpsc::UnboundedSender;
use super::order_cleanup;
use super::filled_order_checker;
use super::position_redemption;
use super::order_placement;
pub async fn run_trading_loop(
    poly_client: PolyClient,
    position_manager: PositionManager,
    pending_order_manager: PendingOrderManager,
    settings: Settings,
    alert_tx: UnboundedSender<String>,
) -> Result<()> {
    println!("Engine ready. Negative Risk Fishing strategy starting...");
    if settings.test_mode {
        println!("TEST MODE ENABLED: Deep limits (95% discount) - orders won't fill immediately");
        println!("Purpose: Verify order placement on Polymarket website");
        println!("To disable: Set TEST_MODE=false in .env or remove the variable");
    }
    println!("Strategy: Place limit orders below market, wait for panic sellers to fill");
    println!("Target: 2% profit per complete set ($100/day requires $5k capital or 5x turnover)");
    let mut cycle_count = 0u32;
    loop {
        cycle_count += 1;
        println!("\n--- New Scan Cycle (#{}) ---", cycle_count);
        if cycle_count % 10 == 0 {
            display_periodic_status(&settings, &position_manager, &pending_order_manager).await;
        }
        position_manager.cleanup_expired().await;
        pending_order_manager.cleanup_expired().await;
        order_cleanup::cancel_orders_near_expiry(&poly_client, &pending_order_manager, &settings).await?;
        filled_order_checker::check_and_convert_filled_orders(
            &poly_client, &pending_order_manager, &position_manager, &settings
        ).await?;
        position_redemption::redeem_profitable_positions(&position_manager, &settings).await;
        let pending_count = pending_order_manager.count().await;
        if pending_count >= 50 {
            println!("Max pending orders reached ({}), skipping new orders", pending_count);
        } else {
            let markets = match poly_client.get_active_btc_markets().await {
                Ok(m) => {
                    if m.is_empty() {
                        println!("No markets found, waiting...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        continue;
                    }
                    m
                }
                Err(e) => {
                    eprintln!("Error fetching markets: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            };
            order_placement::place_fishing_orders(
                &poly_client, &pending_order_manager, &position_manager, &settings,
                markets, &alert_tx
            ).await?;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
async fn display_periodic_status(
    settings: &Settings,
    position_manager: &PositionManager,
    pending_order_manager: &PendingOrderManager,
) {
    println!("Periodic status check...");
    match crate::clients::chain::check_balance(settings).await {
        Ok(balance) => {
            println!("USDC: ${:.2} | POL: {:.4}", balance.usdc, balance.pol);
        }
        Err(e) => {
            eprintln!("Balance check failed: {}", e);
        }
    }
    let position_count = position_manager.count().await;
    let pending_count = pending_order_manager.count().await;
    println!("Active positions: {} | Pending orders: {}", position_count, pending_count);
}
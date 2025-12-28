use anyhow::Result;
use crate::config::settings::Settings;
use crate::engine::pending_orders::PendingOrderManager;
pub async fn check_safety_limits(
    settings: &Settings,
    pending_order_manager: &PendingOrderManager,
    total_order_value: f64,
) -> Result<bool> {
    if settings.one_market_at_a_time {
        let active_markets = pending_order_manager.get_all_markets().await;
        if !active_markets.is_empty() {
            println!("ONE_MARKET_AT_A_TIME: Already trading {} market(s), skipping", active_markets.len());
            return Ok(false);
        }
    }
    if let Some(max_usdc) = settings.max_usdc_per_market {
        if total_order_value > max_usdc {
            println!("MAX_USDC_PER_MARKET: Order value ${:.2} exceeds limit ${:.2}, skipping",
                     total_order_value, max_usdc);
            return Ok(false);
        }
    }
    if let Some(max_percent) = settings.max_wallet_percent_per_order {
        match crate::clients::chain::check_balance(settings).await {
            Ok(balance) => {
                let max_allowed = balance.usdc * max_percent;
                if total_order_value > max_allowed {
                    println!("MAX_WALLET_PERCENT: Order value ${:.2} exceeds {:.1}% of wallet (${:.2}), skipping",
                             total_order_value, max_percent * 100.0, max_allowed);
                    return Ok(false);
                }
            }
            Err(e) => {
                eprintln!("Could not check balance for safety check: {}", e);
            }
        }
    }
    Ok(true)
}
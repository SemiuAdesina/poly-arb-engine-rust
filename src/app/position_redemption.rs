use crate::engine::positions::PositionManager;
use crate::config::settings::Settings;
pub async fn redeem_profitable_positions(
    position_manager: &PositionManager,
    settings: &Settings,
) {
    let positions_to_redeem = position_manager.get_positions_to_exit().await;
    for position in positions_to_redeem {
        if !crate::engine::fishing::should_redeem_filled_set(position.entry_cost) {
            continue;
        }
        println!("Position profitable for redemption: Cost ${:.4}, can redeem for $1.00 (profit: ${:.4})",
                 position.entry_cost, 1.0 - position.entry_cost);
        println!("🔒 Verifying token balances before redemption...");
        let yes_balance_result = crate::clients::chain::check_token_balance(settings, &position.token_id_yes).await;
        let no_balance_result = crate::clients::chain::check_token_balance(settings, &position.token_id_no).await;
        match (yes_balance_result, no_balance_result) {
            (Ok(yes_balance), Ok(no_balance)) => {
                let min_required = 0.999;
                let has_yes = yes_balance >= min_required;
                let has_no = no_balance >= min_required;
                println!("Token balances: YES={:.4}, NO={:.4} (required: >= {:.4} each)",
                         yes_balance, no_balance, min_required);
                if has_yes && has_no {
                    println!("Balance confirmed! Have complete set (YES + NO)");
                    if position.is_too_close_to_expiry() {
                        println!("Position too close to expiry ({} seconds left) - attempting redemption",
                                 position.seconds_until_expiry());
                        println!("ℹ Note: Auto-redemption not yet implemented. Position will expire normally.");
                    } else {
                        println!("⏳ Position still has time ({} seconds until expiry) - holding for now",
                                 position.seconds_until_expiry());
                    }
                } else {
                    println!("INSUFFICIENT BALANCE: YES={:.4}, NO={:.4} (need >= {:.4} each)",
                             yes_balance, no_balance, min_required);
                    println!("Phantom position detected! Removing from tracking to prevent redemption attempts.");
                    println!("This can happen if orders didn't actually fill or tokens were transferred out.");
                    position_manager.remove_position(&position.condition_id).await;
                }
            }
            (Err(e1), Err(e2)) => {
                eprintln!("Failed to verify balances: YES={}, NO={}", e1, e2);
                println!("Skipping redemption - will retry on next cycle");
            }
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("Failed to verify one balance: {}", e);
                println!("Skipping redemption - will retry on next cycle");
            }
        }
    }
    cleanup_expired_positions(position_manager).await;
}
async fn cleanup_expired_positions(position_manager: &PositionManager) {
    let all_positions = position_manager.get_positions().await;
    for position in all_positions {
        if position.is_expired() {
            println!("Position expired: {} (entry: YES ${:.4} + NO ${:.4} = ${:.4})",
                     position.market_question,
                     position.entry_yes_price,
                     position.entry_no_price,
                     position.entry_cost);
            position_manager.remove_position(&position.condition_id).await;
        }
    }
}
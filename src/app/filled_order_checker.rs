use anyhow::Result;
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
use crate::engine::pending_orders::PendingOrderManager;
use crate::engine::positions::PositionManager;
use std::collections::HashMap;
use crate::types::pending_order::PendingOrder;
pub async fn check_and_convert_filled_orders(
    poly_client: &PolyClient,
    pending_order_manager: &PendingOrderManager,
    position_manager: &PositionManager,
    settings: &Settings,
) -> Result<()> {
    let all_pending_orders = pending_order_manager.get_orders().await;
    let mut orders_by_market: HashMap<String, (Option<&PendingOrder>, Option<&PendingOrder>)> = HashMap::new();
    for order in &all_pending_orders {
        let entry = orders_by_market.entry(order.condition_id.clone()).or_insert((None, None));
        if order.side == "YES" {
            entry.0 = Some(order);
        } else if order.side == "NO" {
            entry.1 = Some(order);
        }
    }
    for (condition_id, (yes_order_opt, no_order_opt)) in orders_by_market {
        if let (Some(yes_order), Some(no_order)) = (yes_order_opt, no_order_opt) {
            let existing_positions = position_manager.get_positions().await;
            if existing_positions.iter().any(|p| p.condition_id == condition_id) {
                continue;
            }
            println!("Found complete set of orders for market: {}", yes_order.market_question);
            println!("Verifying fills by checking wallet balances...");
            let yes_balance_result = crate::clients::chain::check_token_balance(settings, &yes_order.token_id).await;
            let no_balance_result = crate::clients::chain::check_token_balance(settings, &no_order.token_id).await;
            let (yes_balance, no_balance) = match (yes_balance_result, no_balance_result) {
                (Ok(yes_bal), Ok(no_bal)) => (yes_bal, no_bal),
                (Err(e1), Err(e2)) => {
                    eprintln!("Failed to check balances: YES={}, NO={}", e1, e2);
                    println!("⏳ Will retry balance check on next cycle");
                    continue;
                }
                (Err(e), _) | (_, Err(e)) => {
                    eprintln!("Failed to check one balance: {}", e);
                    println!("⏳ Will retry balance check on next cycle");
                    continue;
                }
            };
            println!("Token balances: YES={:.4} shares, NO={:.4} shares", yes_balance, no_balance);
            let min_required = yes_order.size - 0.001;
            let yes_filled = yes_balance >= min_required;
            let no_filled = no_balance >= min_required;
            if yes_filled && no_filled {
                println!("CONFIRMED: Both orders filled! YES={:.4}, NO={:.4} (required: {:.4} each)",
                         yes_balance, no_balance, yes_order.size);
                let actual_yes_size = yes_balance.min(yes_order.size);
                let actual_no_size = no_balance.min(no_order.size);
                let position = crate::types::position::Position::new(
                    condition_id.clone(),
                    yes_order.market_question.clone(),
                    yes_order.token_id.clone(),
                    no_order.token_id.clone(),
                    yes_order.expiry_time,
                    yes_order.order_id.clone(),
                    no_order.order_id.clone(),
                    actual_yes_size,
                    actual_no_size,
                    yes_order.limit_price,
                    no_order.limit_price,
                );
                println!("Position details:");
                println!("YES: {} shares of token {} @ ${:.4}",
                         position.size_yes, &position.token_id_yes[..16], position.entry_yes_price);
                println!("NO: {} shares of token {} @ ${:.4}",
                         position.size_no, &position.token_id_no[..16], position.entry_no_price);
                println!("Order IDs: YES {} | NO {}",
                         &position.yes_order_id[..16], &position.no_order_id[..16]);
                println!("Created at: {}, Expires: {}",
                         position.created_at.format("%H:%M:%S"),
                         position.expiry_time.format("%H:%M:%S"));
                position_manager.add_position(position).await;
                pending_order_manager.remove_order(&yes_order.order_id).await;
                pending_order_manager.remove_order(&no_order.order_id).await;
                let total_entry = yes_order.limit_price + no_order.limit_price;
                println!("Position created! Entry cost: ${:.4}, Expected value: $1.00, Profit: ${:.4}",
                         total_entry, 1.0 - total_entry);
            } else {
                if yes_filled || no_filled {
                    println!("PARTIAL FILL: YES filled={}, NO filled={} - waiting for complete fill",
                             yes_filled, no_filled);
                } else {
                    check_orderbook_hints(poly_client, yes_order, no_order).await;
                }
            }
        }
    }
    Ok(())
}
async fn check_orderbook_hints(
    poly_client: &PolyClient,
    yes_order: &PendingOrder,
    no_order: &PendingOrder,
) {
    let yes_ob_result = poly_client.get_orderbook(&yes_order.token_id).await;
    let no_ob_result = poly_client.get_orderbook(&no_order.token_id).await;
    match (yes_ob_result, no_ob_result) {
        (Ok(yes_ob), Ok(no_ob)) => {
            let yes_price_hint = yes_ob.bids.first()
                .and_then(|bid| bid.price.parse::<f64>().ok())
                .map(|bid_price| bid_price >= yes_order.limit_price)
                .unwrap_or(false);
            let no_price_hint = no_ob.bids.first()
                .and_then(|bid| bid.price.parse::<f64>().ok())
                .map(|bid_price| bid_price >= no_order.limit_price)
                .unwrap_or(false);
            if yes_price_hint || no_price_hint {
                println!("Price hint suggests fill possible, but balance check shows no tokens yet");
                println!("⏳ Waiting for blockchain confirmation (may take a few seconds)");
            } else {
                println!("⏳ Orders still pending - no fills detected (price or balance)");
            }
        }
        _ => {
            println!("⏳ Orders still pending - waiting for fills...");
        }
    }
}
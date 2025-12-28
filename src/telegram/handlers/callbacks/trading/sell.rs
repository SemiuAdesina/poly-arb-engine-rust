use teloxide::prelude::*;
use crate::telegram::keyboard::main_menu_keyboard;
use crate::telegram::handlers::constants::EngineState;
use crate::config::settings::Settings;
pub async fn handle_force_sell(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
    engine_state: EngineState,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id).text("Processing sell order...").await?;
    let settings = match Settings::new() {
        Ok(s) => s,
        Err(e) => {
            bot.send_message(chat_id, format!("Configuration error: {}", e))
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
            return Ok(());
        }
    };
    if !settings.manual_trading_enabled {
        bot.send_message(chat_id, "Manual trading is disabled. Set MANUAL_TRADING_ENABLED=true in .env to enable.")
            .reply_markup(main_menu_keyboard(*engine_state.read().await))
            .await?;
        return Ok(());
    }
    let balance_info = match crate::clients::chain::check_balance(&settings).await {
        Ok(b) => b,
        Err(e) => {
            bot.send_message(chat_id, format!("Error checking balance: {}", e))
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
            return Ok(());
        }
    };
    let api_client = crate::clients::poly_api::PolyClient::new(&settings);
    let markets = match api_client.get_active_btc_markets().await {
        Ok(m) => m,
        Err(e) => {
            bot.send_message(chat_id, format!("Error fetching markets: {}", e))
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
            return Ok(());
        }
    };
    let target_market = match crate::engine::discovery::find_best_expiry(markets) {
        Some(m) => m,
        None => {
            bot.send_message(chat_id, "No suitable market found. Try again later.")
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
            return Ok(());
        }
    };
    let shares_to_sell = 10.0;
    let sell_price = 0.50;
    match crate::execution::trade::execute_manual_sell(
        &settings,
        &target_market.condition_id,
        shares_to_sell,
        sell_price,
    ).await {
        Ok(order_id) => {
            let success_msg = format!("Manual Sell Order Executed!\n\nMarket: {}\nShares: {:.4}\nPrice: ${:.4}\n\nTransaction Hash / Order ID:\n`{}`\n\n Current Balance:\nUSDC: ${:.2}\nPOL: {:.4}",
                target_market.question,
                shares_to_sell,
                sell_price,
                order_id,
                balance_info.usdc,
                balance_info.pol
            );
            bot.send_message(chat_id, success_msg)
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
        }
        Err(e) => {
            let error_msg = format!("Sell order failed: {}\n\nThis may be because:\n1. Polymarket API integration is not yet complete\n2. No positions to sell\n3. Market closed\n\nBalance unchanged.",
                e
            );
            bot.send_message(chat_id, error_msg)
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
        }
    }
    Ok(())
}
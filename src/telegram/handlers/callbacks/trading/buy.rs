use teloxide::prelude::*;
use crate::telegram::keyboard::main_menu_keyboard;
use crate::telegram::handlers::constants::EngineState;
use crate::config::settings::Settings;
pub async fn handle_force_buy(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
    engine_state: EngineState,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id).text("Processing buy order...").await?;
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
    if balance_info.usdc < 1.0 {
        bot.send_message(chat_id, format!("Insufficient balance. Need $1 USDC, but you have ${:.2} USDC.",
            balance_info.usdc
        ))
        .reply_markup(main_menu_keyboard(*engine_state.read().await))
        .await?;
        return Ok(());
    }
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
    let buy_price = 0.50;
    let usdc_amount = 1.0;
    match crate::execution::trade::execute_manual_buy(
        &settings,
        &target_market.condition_id,
        usdc_amount,
        buy_price,
    ).await {
        Ok(order_id) => {
            let success_msg = format!("Manual Buy Order Executed!\n\nMarket: {}\nAmount: ${:.2} USDC\nPrice: ${:.4}\n\nTransaction Hash / Order ID:\n`{}`\n\n Remaining Balance:\nUSDC: ${:.2}\nPOL: {:.4}",
                target_market.question,
                usdc_amount,
                buy_price,
                order_id,
                balance_info.usdc - usdc_amount,
                balance_info.pol
            );
            bot.send_message(chat_id, success_msg)
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
        }
        Err(e) => {
            let error_msg = format!("Buy order failed: {}\n\nThis may be because:\n1. Polymarket API integration is not yet complete\n2. Insufficient liquidity\n3. Market closed\n\nBalance remains unchanged.",
                e
            );
            bot.send_message(chat_id, error_msg)
                .reply_markup(main_menu_keyboard(*engine_state.read().await))
                .await?;
        }
    }
    Ok(())
}
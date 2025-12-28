use teloxide::prelude::*;
use crate::telegram::keyboard::{main_menu_keyboard, help_keyboard};
use crate::telegram::handlers::constants::{WELCOME_MESSAGE, EngineState};
use crate::config::settings::Settings;
pub async fn handle_toggle(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
    engine_state: EngineState,
) -> ResponseResult<()> {
    let mut state = engine_state.write().await;
    *state = !*state;
    let status = if *state { "STARTED" } else { "STOPPED" };
    let emoji = if *state { "" } else { "" };
    let msg = format!("{} Engine {}!", emoji, status);
    bot.answer_callback_query(q.id)
        .text(&msg)
        .await?;
    bot.edit_message_reply_markup(chat_id, q.message.unwrap().id)
        .reply_markup(main_menu_keyboard(*state))
        .await?;
    Ok(())
}
pub async fn handle_balance_callback(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
    engine_state: EngineState,
) -> ResponseResult<()> {
    let settings = load_settings();
    let balance_msg = match crate::clients::chain::check_balance(&settings).await {
        Ok(balance) => {
            format!("Wallet Balance:\n\nUSDC: ${:.2}\nPOL: {:.4}",
                balance.usdc, balance.pol
            )
        }
        Err(e) => {
            format!("Error fetching balance: {}\n\nPlease check your RPC_URL and PRIVATE_KEY in .env file.", e)
        }
    };
    bot.answer_callback_query(q.id).await?;
    let is_running = *engine_state.read().await;
    bot.send_message(chat_id, balance_msg)
        .reply_markup(main_menu_keyboard(is_running))
        .await?;
    Ok(())
}
pub async fn handle_stop_callback(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
    engine_state: EngineState,
) -> ResponseResult<()> {
    *engine_state.write().await = false;
    let stop_msg = "🛑 Trading engine has been STOPPED.\n\nAll trading operations are paused. Use /start to resume.";
    bot.answer_callback_query(q.id)
        .text("Engine stopped")
        .await?;
    bot.send_message(chat_id, stop_msg)
        .reply_markup(main_menu_keyboard(false))
        .await?;
    Ok(())
}
pub async fn handle_help_callback(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id).await?;
    bot.send_message(chat_id, WELCOME_MESSAGE)
        .reply_markup(help_keyboard())
        .await?;
    Ok(())
}
pub async fn handle_status(
    bot: Bot,
    q: CallbackQuery,
    chat_id: ChatId,
    engine_state: EngineState,
) -> ResponseResult<()> {
    let is_running = *engine_state.read().await;
    bot.answer_callback_query(q.id).await?;
    let status_msg = format!("Engine Status: {}\n\nUse the buttons below to control the engine.",
        if is_running { "RUNNING" } else { "STOPPED" }
    );
    bot.send_message(chat_id, status_msg)
        .reply_markup(main_menu_keyboard(is_running))
        .await?;
    Ok(())
}
fn load_settings() -> Settings {
    Settings::new().unwrap_or_else(|_| {
        Settings {
            rpc_url: String::new(),
            ws_url: String::new(),
            private_key: String::new(),
            poly_api_key: String::new(),
            poly_api_secret: String::new(),
            poly_passphrase: String::new(),
            telegram_bot_token: String::new(),
            telegram_chat_id: None,
            manual_trading_enabled: true,
            cloudflare_cookie: None,
            flaresolverr_url: None,
            flaresolverr_proxy: None,
            test_mode: false,
            max_usdc_per_market: None,
            max_wallet_percent_per_order: None,
            one_market_at_a_time: false,
            min_profit_pct: 0.2,
        }
    })
}
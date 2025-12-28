use teloxide::prelude::*;
use crate::telegram::keyboard::main_menu_keyboard;
use crate::telegram::handlers::constants::{WELCOME_MESSAGE, EngineState};
use crate::config::settings::Settings;
pub async fn message_handler(
    bot: Bot,
    msg: Message,
    engine_state: EngineState,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        match text {
            "/start" => handle_start(bot, msg, engine_state).await?,
            "/balance" => handle_balance(bot, msg, engine_state).await?,
            "/stop" => handle_stop(bot, msg, engine_state).await?,
            "/help" => handle_help(bot, msg).await?,
            _ => handle_unknown(bot, msg, engine_state).await?,
        }
    }
    Ok(())
}
async fn handle_start(bot: Bot, msg: Message, engine_state: EngineState) -> ResponseResult<()> {
    let welcome = format!("{}\n\nUse the buttons below to control the engine:", WELCOME_MESSAGE);
    let is_running = *engine_state.read().await;
    bot.send_message(msg.chat.id, welcome)
        .reply_markup(main_menu_keyboard(is_running))
        .await?;
    Ok(())
}
async fn handle_balance(bot: Bot, msg: Message, engine_state: EngineState) -> ResponseResult<()> {
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
    let is_running = *engine_state.read().await;
    bot.send_message(msg.chat.id, balance_msg)
        .reply_markup(main_menu_keyboard(is_running))
        .await?;
    Ok(())
}
async fn handle_stop(bot: Bot, msg: Message, engine_state: EngineState) -> ResponseResult<()> {
    *engine_state.write().await = false;
    let stop_msg = "🛑 Trading engine has been STOPPED.\n\nAll trading operations are paused. Use /start to resume.";
    bot.send_message(msg.chat.id, stop_msg)
        .reply_markup(main_menu_keyboard(false))
        .await?;
    Ok(())
}
async fn handle_help(bot: Bot, msg: Message) -> ResponseResult<()> {
    use crate::telegram::keyboard::help_keyboard;
    bot.send_message(msg.chat.id, WELCOME_MESSAGE)
        .reply_markup(help_keyboard())
        .await?;
    Ok(())
}
async fn handle_unknown(bot: Bot, msg: Message, engine_state: EngineState) -> ResponseResult<()> {
    let is_running = *engine_state.read().await;
    bot.send_message(
        msg.chat.id,
        "Unknown command. Use /start to see the menu or tap the buttons below.",
    )
    .reply_markup(main_menu_keyboard(is_running))
    .await?;
    Ok(())
}
fn load_settings() -> Settings {
    Settings::new().unwrap_or_else(|_| {
        dotenv::dotenv().ok();
        Settings {
            rpc_url: std::env::var("RPC_URL").unwrap_or_default(),
            ws_url: std::env::var("WS_URL").unwrap_or_default(),
            private_key: std::env::var("PRIVATE_KEY").unwrap_or_default(),
            poly_api_key: std::env::var("POLY_API_KEY").unwrap_or_default(),
            poly_api_secret: std::env::var("POLY_API_SECRET").unwrap_or_default(),
            poly_passphrase: std::env::var("POLY_PASSPHRASE").unwrap_or_default(),
            telegram_bot_token: String::new(),
            telegram_chat_id: None,
            manual_trading_enabled: std::env::var("MANUAL_TRADING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cloudflare_cookie: std::env::var("CLOUDFLARE_COOKIE").ok(),
            flaresolverr_url: std::env::var("FLARESOLVERR_URL").ok(),
            flaresolverr_proxy: std::env::var("FLARESOLVERR_PROXY").ok(),
            test_mode: false,
            max_usdc_per_market: None,
            max_wallet_percent_per_order: None,
            one_market_at_a_time: false,
            min_profit_pct: 0.2,
        }
    })
}
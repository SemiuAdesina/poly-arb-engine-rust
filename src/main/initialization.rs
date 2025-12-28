use anyhow::Result;
use crate::config::settings::Settings;
use crate::clients::poly_api::PolyClient;
pub async fn initialize_clients(settings: &Settings) -> PolyClient {
    clients::poly_api::PolyClient::new(settings)
}
pub async fn check_initial_balance(settings: &Settings) {
    println!("\n Checking initial wallet balance...");
    match crate::clients::chain::check_balance(settings).await {
        Ok(balance) => {
            println!("USDC: ${:.2}", balance.usdc);
            println!("POL: {:.4}", balance.pol);
            println!("");
        }
        Err(e) => {
            eprintln!("Warning: Could not fetch balance: {}", e);
            eprintln!("Continuing anyway...\n");
        }
    }
}
pub async fn setup_telegram_bot(
    settings: &Settings,
) -> tokio::sync::mpsc::UnboundedSender<String> {
    let (alert_tx, mut alert_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let telegram_token = if !settings.telegram_bot_token.is_empty() {
        Some(settings.telegram_bot_token.clone())
    } else {
        Settings::get_telegram_token()
    };
    if let Some(bot_token) = telegram_token {
        if !bot_token.is_empty() && bot_token != "mock_token" {
            println!("Starting Telegram bot...");
            let chat_id = settings.telegram_chat_id;
            if let Some(chat_id) = chat_id {
                let startup_bot = crate::telegram::TelegramBot::new(&bot_token);
                if let Err(e) = startup_bot.send_startup_message(chat_id).await {
                    eprintln!("Failed to send startup message: {}", e);
                } else {
                    println!("Startup message sent to Telegram");
                }
            }
            let bot_token_clone = bot_token.clone();
            tokio::spawn(async move {
                let bot = crate::telegram::TelegramBot::new(&bot_token_clone);
                bot.start().await;
            });
            if let Some(chat_id) = settings.telegram_chat_id {
                let notification_bot = crate::telegram::TelegramBot::new(&bot_token);
                tokio::spawn(async move {
                    println!("Trade notification listener started");
                    while let Some(msg) = alert_rx.recv().await {
                        if let Err(e) = notification_bot.send_trade_notification(chat_id, &msg).await {
                            eprintln!("Failed to send trade notification: {}", e);
                        } else {
                            println!("Trade notification sent to Telegram");
                        }
                    }
                });
            }
        } else {
            println!("Telegram bot token not set, running without bot");
        }
    } else {
        println!("Telegram bot token not found in environment, running without bot");
    }
    alert_tx
}
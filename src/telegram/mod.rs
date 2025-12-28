pub mod handlers;
pub mod keyboard;
use teloxide::prelude::*;
pub struct TelegramBot {
    pub bot: Bot,
}
impl TelegramBot {
    pub fn new(token: &str) -> Self {
        TelegramBot {
            bot: Bot::new(token),
        }
    }
    pub async fn start(self) {
        handlers::run_bot(self.bot).await;
    }
    pub async fn send_startup_message(&self, chat_id: i64) -> Result<(), teloxide::RequestError> {
        let message = "System Online. Scanning for Arbitrage...";
        self.bot.send_message(ChatId(chat_id), message)
            .reply_markup(crate::telegram::keyboard::main_menu_keyboard(false))
            .await?;
        Ok(())
    }
    pub async fn send_trade_notification(&self, chat_id: i64, message: &str) -> Result<(), teloxide::RequestError> {
        self.bot.send_message(ChatId(chat_id), message).await?;
        Ok(())
    }
}
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
pub fn main_menu_keyboard(is_running: bool) -> InlineKeyboardMarkup {
    let status_text = if is_running { "RUNNING" } else { "STOPPED" };
    let start_text = if is_running { "Pause" } else { "▶ Start Trading" };
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            format!("Engine Status: {}", status_text),
            "status",
        )],
        vec![
            InlineKeyboardButton::callback(start_text.to_string(), "toggle"),
            InlineKeyboardButton::callback("Check Balance".to_string(), "balance"),
        ],
        vec![
            InlineKeyboardButton::callback("Buy $10 USDC".to_string(), "force_buy"),
            InlineKeyboardButton::callback("Sell All".to_string(), "force_sell"),
        ],
        vec![InlineKeyboardButton::callback(
            "🛑 Stop All Trading".to_string(),
            "stop",
        )],
        vec![InlineKeyboardButton::callback(
            "❓ Help".to_string(),
            "help",
        )],
    ])
}
pub fn help_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔙 Back to Menu".to_string(),
        "menu",
    )]])
}
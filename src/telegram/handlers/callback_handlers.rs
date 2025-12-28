use teloxide::prelude::*;
use crate::telegram::handlers::constants::EngineState;
use super::callbacks;
pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    engine_state: EngineState,
) -> ResponseResult<()> {
    if let Some(data) = q.data.clone() {
        let chat_id = q.message.as_ref().and_then(|m| Some(m.chat.id));
        let q_id = q.id.clone();
        if let Some(chat_id) = chat_id {
            match data.as_str() {
                "toggle" => callbacks::handle_toggle(bot, q, chat_id, engine_state).await?,
                "balance" => callbacks::handle_balance_callback(bot, q, chat_id, engine_state).await?,
                "stop" => callbacks::handle_stop_callback(bot, q, chat_id, engine_state).await?,
                "help" => callbacks::handle_help_callback(bot, q, chat_id).await?,
                "force_buy" => callbacks::handle_force_buy(bot, q, chat_id, engine_state).await?,
                "force_sell" => callbacks::handle_force_sell(bot, q, chat_id, engine_state).await?,
                "menu" | "status" => callbacks::handle_status(bot, q, chat_id, engine_state).await?,
                _ => {
                    bot.answer_callback_query(q_id).text("Unknown action").await?;
                }
            }
        }
    }
    Ok(())
}
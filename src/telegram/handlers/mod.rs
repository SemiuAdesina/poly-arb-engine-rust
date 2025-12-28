pub mod constants;
pub mod message_handlers;
pub mod callback_handlers;
pub mod callbacks;
use teloxide::prelude::*;
use constants::EngineState;
use message_handlers::message_handler;
use callback_handlers::callback_handler;
pub async fn run_bot(bot: Bot) {
    let engine_state: EngineState = std::sync::Arc::new(tokio::sync::RwLock::new(false));
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![engine_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
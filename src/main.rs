mod config;
mod types;
mod clients;
mod engine;
mod execution;
mod utils;
mod telegram;
mod app;
use anyhow::{Result, Context};
use config::settings::Settings;
#[tokio::main]
async fn main() -> Result<()> {
    utils::logger::setup();
    println!("Starting PolyArb Engine v0.1...");
    let settings = Settings::new()
        .context("Failed to load settings from .env file. Please ensure all required environment variables are set")?;
    app::startup::display_startup_banner(&settings)?;
    let poly_client = app::initialization::initialize_clients(&settings).await;
    app::initialization::check_initial_balance(&settings).await;
    let alert_tx = app::initialization::setup_telegram_bot(&settings).await;
    let position_manager = engine::positions::PositionManager::new();
    let pending_order_manager = engine::pending_orders::PendingOrderManager::new();
    app::trading_loop::run_trading_loop(
        poly_client,
        position_manager,
        pending_order_manager,
        settings,
        alert_tx,
    ).await
}
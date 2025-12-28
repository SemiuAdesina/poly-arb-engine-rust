use std::sync::Arc;
use tokio::sync::RwLock;

pub type EngineState = Arc<RwLock<bool>>;

pub const WELCOME_MESSAGE: &str = "Welcome to PolyArb Engine!\n\nThis bot helps you manage your Polymarket arbitrage trading engine.\n\nCommands:\n/start - Start the engine\n/balance - Check wallet balance\n/stop - Stop the engine\n/help - Show help";


use std::env;
use dotenv::dotenv;
use anyhow::{Result, Context};
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Settings {
    pub rpc_url: String,
    pub ws_url: String,
    pub private_key: String,
    pub poly_api_key: String,
    pub poly_api_secret: String,
    pub poly_passphrase: String,
    pub telegram_bot_token: String,
    pub telegram_chat_id: Option<i64>,
    pub manual_trading_enabled: bool,
    pub cloudflare_cookie: Option<String>,
    pub flaresolverr_url: Option<String>, // Optional FlareSolverr URL (defaults to http://localhost:8191)
    pub flaresolverr_proxy: Option<String>, // Optional proxy URL for FlareSolverr (e.g., "http://proxy:port" or "socks5://proxy:port")
    pub test_mode: bool,
    pub max_usdc_per_market: Option<f64>,
    pub max_wallet_percent_per_order: Option<f64>,
    pub one_market_at_a_time: bool,
    pub min_profit_pct: f64,
}
impl Settings {
    pub fn new() -> Result<Self> {
        dotenv().ok();
        Ok(Settings {
            rpc_url: env::var("RPC_URL").context("RPC_URL must be set")?,
            ws_url: env::var("WS_URL").context("WS_URL must be set")?,
            private_key: env::var("PRIVATE_KEY").context("PRIVATE_KEY must be set")?,
            poly_api_key: env::var("POLY_API_KEY").context("POLY_API_KEY must be set")?,
            poly_api_secret: env::var("POLY_API_SECRET").context("POLY_API_SECRET must be set")?,
            poly_passphrase: env::var("POLY_PASSPHRASE").context("POLY_PASSPHRASE must be set")?,
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID").ok().and_then(|s| s.parse().ok()),
            manual_trading_enabled: env::var("MANUAL_TRADING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cloudflare_cookie: env::var("CLOUDFLARE_COOKIE").ok(),
            flaresolverr_url: env::var("FLARESOLVERR_URL").ok(),
            flaresolverr_proxy: env::var("FLARESOLVERR_PROXY").ok(),
            test_mode: env::var("TEST_MODE")
                .ok()
                .map(|s| s.to_lowercase())
                .and_then(|s| match s.as_str() {
                    "true" | "1" | "yes" | "on" => Some(true),
                    "false" | "0" | "no" | "off" => Some(false),
                    _ => s.parse().ok(),
                })
                .unwrap_or(false),
            max_usdc_per_market: env::var("MAX_USDC_PER_MARKET")
                .ok()
                .and_then(|s| s.parse().ok())
                .filter(|&v| v > 0.0),
            max_wallet_percent_per_order: env::var("MAX_WALLET_PERCENT_PER_ORDER")
                .ok()
                .and_then(|s| s.parse().ok())
                .filter(|&v| v > 0.0 && v <= 1.0),
            one_market_at_a_time: env::var("ONE_MARKET_AT_A_TIME")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            min_profit_pct: env::var("MIN_PROFIT_PCT")
                .ok()
                .and_then(|s| s.parse().ok())
                .filter(|&v| v >= 0.0)
                .unwrap_or(0.2),
        })
    }
    pub fn get_telegram_token() -> Option<String> {
        dotenv().ok();
        env::var("TELEGRAM_BOT_TOKEN").ok()
    }
}
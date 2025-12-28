use crate::config::settings::Settings;
use chrono::Utc;
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BOLD: &str = "\x1b[1m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
}
pub fn display_trading_mode_banner_with_settings(settings: &Settings, log_entries: &mut Vec<String>) -> String {
    let dry_run = std::env::var("DRY_RUN")
        .ok()
        .map(|s| s.to_lowercase())
        .and_then(|s| match s.as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            _ => s.parse().ok(),
        })
        .unwrap_or(false);
    let test_mode_env = std::env::var("TEST_MODE").ok();
    let test_mode_value = test_mode_env.as_ref().map(|s| s.as_str()).unwrap_or("");
    log_entries.push("=== PolyArb Engine Startup ===".to_string());
    log_entries.push(format!("Timestamp: {}", Utc::now().to_rfc3339()));
    println!("\n{}", "█".repeat(70));
    let trading_mode = if dry_run {
        log_entries.push("Mode: DRY_RUN (safe testing)".to_string());
        println!("{}{}█{}", colors::CYAN, colors::BOLD, colors::RESET);
        println!("{}{} DRY_RUN MODE: Orders will NOT be placed (safe testing){}{}",
                 colors::CYAN, colors::BOLD, colors::RESET, colors::RESET);
        println!("{}{}█{}", colors::CYAN, colors::BOLD, colors::RESET);
        "DRY_RUN"
    } else if settings.test_mode {
            log_entries.push("Mode: TEST (deep limits, real orders sent)".to_string());
            println!("{}{}█{}", colors::YELLOW, colors::BOLD, colors::RESET);
            println!("{}{} TEST MODE ENABLED{}{}", colors::YELLOW, colors::BOLD, colors::RESET, colors::RESET);
            println!("Deep limits (95% discount) - orders won't fill immediately");
            println!("{}{} WARNING: Real orders WILL be sent to Polymarket!{}{}",
                     colors::BRIGHT_YELLOW, colors::BOLD, colors::RESET, colors::RESET);
            if !test_mode_value.is_empty() {
                println!("TEST_MODE={} (set in environment)", test_mode_value);
                log_entries.push(format!("TEST_MODE env var: {}", test_mode_value));
            }
            println!("{}{}█{}", colors::YELLOW, colors::BOLD, colors::RESET);
            "TEST"
        } else {
            log_entries.push("Mode: LIVE TRADING (real orders, real money)".to_string());
            println!("{}{}█{}", colors::BRIGHT_RED, colors::BOLD, colors::RESET);
            println!("{}{} LIVE TRADING MODE {}{}",
                     colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
            println!("{}{} Real orders WILL be placed on Polymarket{}{}",
                     colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
            println!("{}{} Real money WILL be at risk{}{}",
                     colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
            if !test_mode_value.is_empty() && test_mode_value.to_lowercase() != "false" {
                println!();
                eprintln!("{}{} WARNING: TEST_MODE is set to '{}' but parsed as false{}{}",
                         colors::BRIGHT_YELLOW, colors::BOLD, test_mode_value, colors::RESET, colors::RESET);
                eprintln!("To disable test mode: unset TEST_MODE");
                eprintln!("Or set: export TEST_MODE=false");
                log_entries.push(format!("WARNING: TEST_MODE env var set to '{}' but parsed as false", test_mode_value));
            }
            println!("{}{}█{}", colors::BRIGHT_RED, colors::BOLD, colors::RESET);
            "LIVE"
        };
    trading_mode.to_string()
}
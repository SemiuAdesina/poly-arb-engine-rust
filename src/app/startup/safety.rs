use anyhow::Result;
use crate::config::settings::Settings;
use std::fs;
use chrono::Utc;
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const GREEN: &str = "\x1b[32m";
    pub const BOLD: &str = "\x1b[1m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const CYAN: &str = "\x1b[36m";
}
pub fn display_safety_settings(settings: &Settings, log_entries: &mut Vec<String>) {
    println!();
    println!("{}{} SAFETY SETTINGS:{}{}", colors::GREEN, colors::BOLD, colors::RESET, colors::RESET);
    if let Some(max_usdc) = settings.max_usdc_per_market {
        println!("Max USDC per market: ${:.2}", max_usdc);
        log_entries.push(format!("Max USDC per market: ${:.2}", max_usdc));
    } else {
        println!("{}{} Max USDC per market: NOT SET (unlimited risk!){}{}",
                 colors::BRIGHT_YELLOW, colors::BOLD, colors::RESET, colors::RESET);
        log_entries.push("Max USDC per market: NOT SET".to_string());
    }
    if let Some(max_pct) = settings.max_wallet_percent_per_order {
        println!("Max wallet % per order: {:.1}%", max_pct * 100.0);
        log_entries.push(format!("Max wallet % per order: {:.1}%", max_pct * 100.0));
    } else {
        println!("{}{} Max wallet % per order: NOT SET (unlimited risk!){}{}",
                 colors::BRIGHT_YELLOW, colors::BOLD, colors::RESET, colors::RESET);
        log_entries.push("Max wallet % per order: NOT SET".to_string());
    }
    if settings.one_market_at_a_time {
        println!("One market at a time: ENABLED");
        log_entries.push("One market at a time: ENABLED".to_string());
    } else {
        println!("One market at a time: DISABLED (multiple markets allowed)");
        log_entries.push("One market at a time: DISABLED".to_string());
    }
    println!("Profit threshold: VARIED [0.8%, 1.5%, 2.0%, 2.5%] (randomly selected per order)");
    log_entries.push("Profit threshold: VARIED [0.8%, 1.5%, 2.0%, 2.5%] (randomly selected per order)".to_string());
    println!();
    println!("{}", "█".repeat(70));
    println!();
}
pub fn write_startup_log(log_entries: &[String], settings: &Settings) -> Result<()> {
    let log_dir = "logs";
    if let Err(e) = fs::create_dir_all(log_dir) {
        eprintln!("Could not create logs directory: {}", e);
        return Ok(());
    }
    let log_file = format!("{}/startup_{}.log", log_dir, Utc::now().format("%Y%m%d_%H%M%S"));
    if let Ok(mut file) = fs::File::create(&log_file) {
        for entry in log_entries {
            use std::io::Write;
            writeln!(file, "{}", entry).ok();
        }
        use std::io::Write;
        writeln!(file, "").ok();
        writeln!(file, "FlareSolverr URL: {:?}", settings.flaresolverr_url).ok();
        writeln!(file, "Wallet address: (from PRIVATE_KEY)").ok();
        println!("{}{} Startup log saved to: {}{}{}",
                 colors::CYAN, colors::BOLD, log_file, colors::RESET, colors::RESET);
    }
    Ok(())
}
pub fn confirm_live_trading() -> Result<()> {
    use std::io::{self, Write};
    mod colors {
        pub const RESET: &str = "\x1b[0m";
        pub const BRIGHT_RED: &str = "\x1b[91m";
        pub const BOLD: &str = "\x1b[1m";
        pub const GREEN: &str = "\x1b[32m";
    }
    println!("{}{} CONFIRMATION REQUIRED FOR LIVE TRADING{}{}",
             colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
    print!("{}{}Type 'YES' to confirm live trading mode: {}{}",
           colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
    io::stdout().flush().ok();
    let mut confirmation = String::new();
    if io::stdin().read_line(&mut confirmation).is_ok() {
        let trimmed = confirmation.trim();
        if trimmed != "YES" {
            println!("{}{} Confirmation failed. Exiting for safety.{}{}",
                     colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
            return Err(anyhow::anyhow!("Live trading mode not confirmed. Exiting for safety."));
        }
        println!("{}{} Live trading mode confirmed. Starting bot...{}{}\n",
                 colors::GREEN, colors::BOLD, colors::RESET, colors::RESET);
    } else {
        println!("{}{} Could not read confirmation. Exiting for safety.{}{}",
                 colors::BRIGHT_RED, colors::BOLD, colors::RESET, colors::RESET);
        return Err(anyhow::anyhow!("Could not read confirmation. Exiting for safety."));
    }
    Ok(())
}
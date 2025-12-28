pub mod banner;
pub mod safety;
use anyhow::Result;
use crate::config::settings::Settings;
pub fn display_startup_banner(settings: &Settings) -> Result<String> {
    let mut log_entries = Vec::new();
    let trading_mode = banner::display_trading_mode_banner_with_settings(settings, &mut log_entries);
    safety::display_safety_settings(settings, &mut log_entries);
    safety::write_startup_log(&log_entries, settings)?;
    if trading_mode == "LIVE" {
        safety::confirm_live_trading()?;
    }
    Ok(trading_mode)
}
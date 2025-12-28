use crate::types::market::Market;
use chrono::{DateTime, Utc};
pub fn find_best_expiry(markets: Vec<Market>) -> Option<Market> {
    println!("--> Finding best market (any non-expired market, prefer soonest)...");
    println!("Debug: Processing {} markets", markets.len());
    let now = Utc::now();
    let mut best_market: Option<Market> = None;
    let mut shortest_time_until_expiry = std::time::Duration::MAX;
    let mut valid_markets = 0;
    let mut expired_markets = 0;
    let mut invalid_date_markets = 0;
    for (idx, market) in markets.iter().enumerate() {
        if market.end_date_iso.is_empty() {
            invalid_date_markets += 1;
            if idx < 3 {
                eprintln!("Market {}: Empty expiry date - {}", idx + 1, market.question);
            }
            continue;
        }
        match DateTime::parse_from_rfc3339(&market.end_date_iso) {
            Ok(expiry_time) => {
                let expiry_utc = expiry_time.with_timezone(&Utc);
                if expiry_utc > now {
                    valid_markets += 1;
                    let time_until_expiry = expiry_utc - now;
                    let time_until_expiry_secs = time_until_expiry.num_seconds();
                    if time_until_expiry_secs <= 0 {
                        expired_markets += 1;
                        continue;
                    }
                    let time_until_expiry_duration = std::time::Duration::from_secs(time_until_expiry_secs as u64);
                    if valid_markets <= 3 {
                        let days = time_until_expiry_secs / 86400;
                        let hours = (time_until_expiry_secs % 86400) / 3600;
                        println!("✓ Market {}: '{}' expires in {}d {}h (condition_id: {})",
                            idx + 1, market.question, days, hours, &market.condition_id[..16]);
                    }
                    if time_until_expiry_duration < shortest_time_until_expiry {
                        shortest_time_until_expiry = time_until_expiry_duration;
                        best_market = Some(market.clone());
                    }
                } else {
                    expired_markets += 1;
                }
            }
            Err(e) => {
                invalid_date_markets += 1;
                if idx < 3 {
                    eprintln!("Market {}: Failed to parse expiry '{}' - {}: {}",
                        idx + 1, market.end_date_iso, market.question, e);
                }
            }
        }
    }
    println!("Debug summary: {} valid, {} expired, {} invalid dates",
        valid_markets, expired_markets, invalid_date_markets);
    if let Some(ref market) = best_market {
        println!("Found best market: {}", market.question);
        if let Ok(expiry) = DateTime::parse_from_rfc3339(&market.end_date_iso) {
            let expiry_utc = expiry.with_timezone(&Utc);
            let hours_until_expiry = (expiry_utc - now).num_hours();
            let mins_until_expiry = (expiry_utc - now).num_minutes() % 60;
            let days_until_expiry = (expiry_utc - now).num_days();
            if days_until_expiry > 0 {
                println!("Expires in: {} days {} hours", days_until_expiry, hours_until_expiry % 24);
            } else {
                println!("Expires in: {} hours {} minutes", hours_until_expiry, mins_until_expiry);
            }
        }
    } else {
        println!("No non-expired markets found");
    }
    best_market
}
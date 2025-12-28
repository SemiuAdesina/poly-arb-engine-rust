use crate::types::market::Market;
use chrono::{DateTime, Utc};
pub fn find_all_valid_markets(markets: Vec<Market>) -> Vec<Market> {
    let now = Utc::now();
    let mut valid_markets = Vec::new();
    let min_seconds = 3600;
    let max_seconds = 365 * 24 * 3600;
    for market in markets {
        if market.end_date_iso.is_empty() {
            continue;
        }
        if let Ok(expiry_time) = DateTime::parse_from_rfc3339(&market.end_date_iso) {
            let expiry_utc = expiry_time.with_timezone(&Utc);
            if expiry_utc > now {
                let time_until_expiry = expiry_utc - now;
                let time_until_expiry_secs = time_until_expiry.num_seconds();
                if time_until_expiry_secs >= min_seconds && time_until_expiry_secs <= max_seconds {
                    let hours_until_expiry = time_until_expiry_secs as f64 / 3600.0;
                    println!("Valid market: '{}' (expires in {:.1} hours)",
                             market.question, hours_until_expiry);
                    valid_markets.push(market);
                } else {
                    let hours_until_expiry = time_until_expiry_secs as f64 / 3600.0;
                    if time_until_expiry_secs > 0 {
                        println!("Market filtered: '{}' (expires in {:.1} hours, outside {:.1}-{:.1} hour window)",
                                 market.question, hours_until_expiry,
                                 min_seconds as f64 / 3600.0, max_seconds as f64 / 3600.0);
                    }
                }
            }
        }
    }
    valid_markets.sort_by(|a, b| {
        let expiry_a = DateTime::parse_from_rfc3339(&a.end_date_iso)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now() + chrono::Duration::days(365));
        let expiry_b = DateTime::parse_from_rfc3339(&b.end_date_iso)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now() + chrono::Duration::days(365));
        expiry_a.cmp(&expiry_b)
    });
    valid_markets
}
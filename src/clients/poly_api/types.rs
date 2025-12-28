#[derive(Clone, Copy, Debug)]
pub enum OutcomeSide {
    Yes,
    No,
}
impl OutcomeSide {
    pub fn from_str_lower(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "yes" => Some(OutcomeSide::Yes),
            "no" => Some(OutcomeSide::No),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            OutcomeSide::Yes => "YES",
            OutcomeSide::No => "NO",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}
impl OrderSide {
    pub fn is_buy(&self) -> bool {
        matches!(self, OrderSide::Buy)
    }
}
use std::sync::atomic::{AtomicU64, Ordering};
pub static LAST_NONCE: AtomicU64 = AtomicU64::new(0);
pub fn get_next_nonce() -> u64 {
    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
    LAST_NONCE
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |prev| {
            Some(prev.max(now_ms) + 1)
        })
        .unwrap_or(now_ms + 1)
}
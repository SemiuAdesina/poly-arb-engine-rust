use chrono::{DateTime, Utc};
#[derive(Debug, Clone)]
pub struct PendingOrder {
    pub condition_id: String,
    pub market_question: String,
    pub token_id: String,
    pub side: String,
    pub order_id: String,
    pub limit_price: f64,
    pub size: f64,
    pub expiry_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
impl PendingOrder {
    pub fn new(
        condition_id: String,
        market_question: String,
        token_id: String,
        side: String,
        order_id: String,
        limit_price: f64,
        size: f64,
        expiry_time: DateTime<Utc>,
    ) -> Self {
        PendingOrder {
            condition_id,
            market_question,
            token_id,
            side,
            order_id,
            limit_price,
            size,
            expiry_time,
            created_at: Utc::now(),
        }
    }
    pub fn should_cancel(&self) -> bool {
        let now = Utc::now();
        let time_until_expiry = self.expiry_time - now;
        time_until_expiry.num_seconds() < 300 || now >= self.expiry_time
    }
}
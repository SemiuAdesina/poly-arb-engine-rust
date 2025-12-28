use chrono::{DateTime, Utc};
#[derive(Debug, Clone)]
pub struct Position {
    pub condition_id: String,
    pub market_question: String,
    pub token_id_yes: String,
    pub token_id_no: String,
    pub expiry_time: DateTime<Utc>,
    pub yes_order_id: String,
    pub no_order_id: String,
    pub size_yes: f64,
    pub size_no: f64,
    pub entry_yes_price: f64,
    pub entry_no_price: f64,
    pub entry_cost: f64,
    pub created_at: DateTime<Utc>,
}
impl Position {
    pub fn new(
        condition_id: String,
        market_question: String,
        token_id_yes: String,
        token_id_no: String,
        expiry_time: DateTime<Utc>,
        yes_order_id: String,
        no_order_id: String,
        size_yes: f64,
        size_no: f64,
        entry_yes_price: f64,
        entry_no_price: f64,
    ) -> Self {
        let entry_cost = (entry_yes_price + entry_no_price) * size_yes;
        Position {
            condition_id,
            market_question,
            token_id_yes,
            token_id_no,
            expiry_time,
            yes_order_id,
            no_order_id,
            size_yes,
            size_no,
            entry_yes_price,
            entry_no_price,
            entry_cost,
            created_at: Utc::now(),
        }
    }
    pub fn should_exit_now(&self) -> bool {
        let now = Utc::now();
        let time_until_expiry = self.expiry_time - now;
        let seconds_until_expiry = time_until_expiry.num_seconds();
        seconds_until_expiry >= 60 && seconds_until_expiry <= 120
    }
    pub fn is_too_close_to_expiry(&self) -> bool {
        let now = Utc::now();
        let time_until_expiry = self.expiry_time - now;
        time_until_expiry.num_seconds() < 60
    }
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expiry_time
    }
    pub fn seconds_until_expiry(&self) -> i64 {
        let now = Utc::now();
        let time_until_expiry = self.expiry_time - now;
        time_until_expiry.num_seconds()
    }
}
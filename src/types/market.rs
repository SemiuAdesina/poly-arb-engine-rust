use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Market {
    pub condition_id: String,
    pub question: String,
    pub token_id_yes: String,
    pub token_id_no: String,
    pub end_date_iso: String,
    #[serde(default)]
    pub enable_orderbook: bool,
}
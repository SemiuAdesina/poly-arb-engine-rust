use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct PriceLevel {
    pub price: String,
    pub size: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}
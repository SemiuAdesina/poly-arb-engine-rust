use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::pending_order::PendingOrder;
pub struct PendingOrderManager {
    orders: Arc<RwLock<Vec<PendingOrder>>>,
}
impl PendingOrderManager {
    pub fn new() -> Self {
        PendingOrderManager {
            orders: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub async fn add_order(&self, order: PendingOrder) {
        let mut orders = self.orders.write().await;
        let age_seconds = (chrono::Utc::now() - order.created_at).num_seconds();
        println!("Added pending order: {} {} @ ${:.4} (order ID: {}, created {}s ago)",
                 order.side, order.market_question, order.limit_price,
                 &order.order_id[..16], age_seconds);
        orders.push(order);
    }
    pub async fn get_orders(&self) -> Vec<PendingOrder> {
        let orders = self.orders.read().await;
        orders.clone()
    }
    pub async fn get_orders_for_market(&self, condition_id: &str) -> Vec<PendingOrder> {
        let orders = self.orders.read().await;
        orders.iter()
            .filter(|o| o.condition_id == condition_id)
            .cloned()
            .collect()
    }
    pub async fn has_complete_set(&self, condition_id: &str) -> bool {
        let orders = self.orders.read().await;
        let market_orders: Vec<_> = orders.iter()
            .filter(|o| o.condition_id == condition_id)
            .collect();
        let has_yes = market_orders.iter().any(|o| o.side == "YES");
        let has_no = market_orders.iter().any(|o| o.side == "NO");
        has_yes && has_no
    }
    pub async fn remove_order(&self, order_id: &str) {
        let mut orders = self.orders.write().await;
        let before = orders.len();
        orders.retain(|o| o.order_id != order_id);
        let after = orders.len();
        if before != after {
            println!("Removed pending order: {}", order_id);
        }
    }
    pub async fn get_orders_to_cancel(&self) -> Vec<PendingOrder> {
        let orders = self.orders.read().await;
        orders.iter()
            .filter(|o| o.should_cancel())
            .cloned()
            .collect()
    }
    pub async fn cleanup_expired(&self) {
        let mut orders = self.orders.write().await;
        let before = orders.len();
        orders.retain(|o| !o.should_cancel());
        let after = orders.len();
        if before != after {
            println!("🧹 Cleaned up {} expired/cancelled pending order(s)", before - after);
        }
    }
    pub async fn count(&self) -> usize {
        let orders = self.orders.read().await;
        orders.len()
    }
    pub async fn get_all_markets(&self) -> Vec<String> {
        let orders = self.orders.read().await;
        let mut markets: std::collections::HashSet<String> = std::collections::HashSet::new();
        for order in orders.iter() {
            markets.insert(order.condition_id.clone());
        }
        markets.into_iter().collect()
    }
}
impl Default for PendingOrderManager {
    fn default() -> Self {
        Self::new()
    }
}
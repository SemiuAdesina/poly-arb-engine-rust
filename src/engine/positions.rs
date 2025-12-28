use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::position::Position;
pub struct PositionManager {
    positions: Arc<RwLock<Vec<Position>>>,
}
impl PositionManager {
    pub fn new() -> Self {
        PositionManager {
            positions: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub async fn add_position(&self, position: Position) {
        let mut positions = self.positions.write().await;
        println!("Added position: {} (expires in {} seconds)",
                 position.market_question, position.seconds_until_expiry());
        positions.push(position);
    }
    pub async fn get_positions(&self) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.clone()
    }
    pub async fn get_positions_to_exit(&self) -> Vec<Position> {
        let positions = self.positions.read().await;
        positions.iter()
            .filter(|pos| pos.should_exit_now() && !pos.is_expired())
            .cloned()
            .collect()
    }
    pub async fn remove_position(&self, condition_id: &str) {
        let mut positions = self.positions.write().await;
        positions.retain(|p| p.condition_id != condition_id);
        println!("Removed position: {}", condition_id);
    }
    pub async fn cleanup_expired(&self) {
        let mut positions = self.positions.write().await;
        let before = positions.len();
        positions.retain(|p| !p.is_expired());
        let after = positions.len();
        if before != after {
            println!("🧹 Cleaned up {} expired position(s)", before - after);
        }
    }
    pub async fn count(&self) -> usize {
        let positions = self.positions.read().await;
        positions.len()
    }
}
impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
}
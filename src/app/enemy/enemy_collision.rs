use crate::framework::types::Vec2I;

// Collision Result
pub enum EnemyCollisionResult {
    NoHit,
    Hit {
        pos: Vec2I,
        destroyed: bool,
        point: u32,
        capturing_player: bool,
    },
}

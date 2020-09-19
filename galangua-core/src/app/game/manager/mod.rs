mod appearance_manager;
mod appearance_table;
mod attack_manager;
mod enemy_manager;
mod event_queue;
pub mod formation;
pub mod game_manager;
pub mod score_holder;

pub use self::enemy_manager::EnemyManager;
pub use self::event_queue::{EventQueue, EventType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CaptureState {
    NoCapture,
    CaptureAttacking,
    Capturing,
    Captured,
    Recapturing,
}

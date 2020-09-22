mod appearance_manager;
mod appearance_table;
mod attack_manager;
mod enemy_manager;
pub mod event_queue;
pub mod formation;
pub mod game_manager;
pub mod score_holder;

pub use self::event_queue::EventType;

#[derive(Clone, Copy, PartialEq)]
pub enum CaptureState {
    NoCapture,
    CaptureAttacking,
    Capturing,
    Captured,
    Recapturing,
    Dual,
}

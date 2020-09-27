mod appearance_manager;
mod appearance_table;
mod attack_manager;
pub mod event_queue;
pub mod formation;
pub mod game_manager;
pub mod score_holder;
mod stage_manager;

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

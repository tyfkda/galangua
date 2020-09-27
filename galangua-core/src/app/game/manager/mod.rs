pub mod event_queue;
pub mod game_manager;
pub mod score_holder;
pub mod stage;

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

pub mod event_queue;
pub mod game_manager;
pub mod stage;

use galangua_common::app::game::FormationIndex;
use galangua_common::framework::types::Vec2I;

#[derive(Clone)]
pub enum CaptureEventType {
    StartCaptureAttack(FormationIndex),
    EndCaptureAttack,
    CapturePlayer(Vec2I),
    CapturePlayerCompleted,
    CaptureSequenceEnded,
    SpawnCapturedFighter(Vec2I, FormationIndex),
    RecapturePlayer(FormationIndex, i32),
    MovePlayerHomePos,
    RecaptureEnded(bool),  // dual succeeded? (false when player died during recapturing)
    EscapeCapturing,
    EscapeEnded,
    CapturedFighterDestroyed,
}

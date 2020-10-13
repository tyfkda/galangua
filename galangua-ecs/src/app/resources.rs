use galangua_common::app::game::{CaptureState, FormationIndex};

pub struct GameInfo {
    pub capture_state: CaptureState,
    pub capture_enemy_fi: FormationIndex,
}

impl GameInfo {
    pub fn new() -> Self {
        GameInfo {
            capture_state: CaptureState::NoCapture,
            capture_enemy_fi: FormationIndex(0, 0),
        }
    }

    pub fn end_capture_attack(&mut self) {
        self.capture_state = CaptureState::NoCapture;
        self.capture_enemy_fi = FormationIndex(0, 0);
    }
}

impl Default for GameInfo {
    fn default() -> Self {
        GameInfo::new()
    }
}

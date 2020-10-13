use galangua_common::app::game::{CaptureState, FormationIndex};

#[derive(PartialEq)]
pub enum GameState {
    //StartStage,
    Playing,
    //PlayerDead,
    //WaitReady,
    //WaitReady2,
    Capturing,
    //Captured,
    //Recapturing,
    //StageClear,
    //GameOver,
    //Finished,
}

pub struct GameInfo {
    pub game_state: GameState,
    pub capture_state: CaptureState,
    pub capture_enemy_fi: FormationIndex,
}

impl GameInfo {
    pub fn new() -> Self {
        GameInfo {
            game_state: GameState::Playing,
            capture_state: CaptureState::NoCapture,
            capture_enemy_fi: FormationIndex(0, 0),
        }
    }

    pub fn can_capture_attack(&self) -> bool {
        self.capture_state == CaptureState::NoCapture
    }

    pub fn can_capture(&self) -> bool {
        self.game_state == GameState::Playing
    }

    pub fn end_capture_attack(&mut self) {
        self.capture_state = CaptureState::NoCapture;
        self.capture_enemy_fi = FormationIndex(0, 0);
    }

    pub fn start_capturing(&mut self) {
        self.capture_state = CaptureState::Capturing;
    }

    pub fn capture_player(&mut self) {
        self.game_state = GameState::Capturing;
        self.capture_state = CaptureState::Capturing;
    }
}

impl Default for GameInfo {
    fn default() -> Self {
        GameInfo::new()
    }
}

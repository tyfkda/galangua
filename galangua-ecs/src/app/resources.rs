use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::{CaptureState, FormationIndex};

#[derive(PartialEq)]
pub enum GameState {
    //StartStage,
    Playing,
    PlayerDead,
    WaitReady,
    WaitReady2,
    Capturing,
    Captured,
    Recapturing,
    //StageClear,
    GameOver,
    //Finished,
}

pub struct GameInfo {
    pub left_ship: u32,
    pub game_state: GameState,
    pub count: u32,
    pub capture_state: CaptureState,
    pub capture_enemy_fi: FormationIndex,
}

pub type GameInfoUpdateParams<'a> = (Write<'a, AttackManager>, Write<'a, StarManager>);

impl GameInfo {
    pub fn new() -> Self {
        GameInfo {
            left_ship: DEFAULT_LEFT_SHIP,
            game_state: GameState::Playing,
            count: 0,
            capture_state: CaptureState::NoCapture,
            capture_enemy_fi: FormationIndex(0, 0),
        }
    }

    pub fn update(&mut self, (mut attack_manager, mut star_manager): GameInfoUpdateParams) {
        match self.game_state {
            GameState::PlayerDead => {
                self.count += 1;
                if self.count >= 60 {
                    self.game_state = GameState::WaitReady;
                    self.count = 0;
                }
            }
            GameState::WaitReady => {
                if attack_manager.is_no_attacker() {
                    self.count += 1;
                    if self.count >= 60 {
                        self.next_player();
                    }
                }
            }
            GameState::WaitReady2 => {
                self.count += 1;
                if self.count >= 60 {
                    //player.set_shot_enable(true);
                    attack_manager.pause(false);
                    star_manager.set_stop(false);
                    self.game_state = GameState::Playing;
                    self.count = 0;
                }
            }
            _ => {}
        }
    }

    pub fn can_capture_attack(&self) -> bool {
        self.capture_state == CaptureState::NoCapture
    }

    pub fn can_capture(&self) -> bool {
        self.game_state == GameState::Playing
    }

    pub fn end_capture_attack(&mut self) {
        assert!(self.capture_state != CaptureState::Captured);
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

    pub fn player_captured(&mut self) {
        self.capture_state = CaptureState::Captured;
        self.game_state = GameState::Captured;
    }

    pub fn crash_player(&mut self, died: bool, attack_manager: &mut AttackManager) {
        if died {
            if self.game_state != GameState::Recapturing {
                attack_manager.pause(true);
                self.game_state = GameState::PlayerDead;
                self.count = 0;
            }
        } else {
            // Must be one of dual fighter crashed.
            assert!(self.capture_state == CaptureState::Dual);
            self.capture_state = CaptureState::NoCapture;
        }
    }

    pub fn next_player(&mut self) {
        self.left_ship -= 1;
        if self.left_ship == 0 {
            //self.stage_manager.pause_attack(true);
            self.game_state = GameState::GameOver;
            self.count = 0;
        } else {
            //self.player.restart();
            //self.player.set_shot_enable(false);
            self.game_state = GameState::WaitReady2;
            self.count = 0;
        }
    }
}

impl Default for GameInfo {
    fn default() -> Self {
        GameInfo::new()
    }
}

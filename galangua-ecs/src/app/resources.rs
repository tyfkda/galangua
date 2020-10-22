use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::stage_indicator::StageIndicator;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::{CaptureState, FormationIndex};
use galangua_common::app::score_holder::ScoreHolder;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::SystemTrait;
use galangua_common::util::math::{atan2_lut, calc_velocity, clamp, ANGLE, ONE};

use super::components::*;
use super::system::system_player::{enable_player_shot, restart_player};

const WAIT1: u32 = 60;

#[derive(PartialEq)]
pub enum GameState {
    StartStage,
    Playing,
    PlayerDead,
    WaitReady,
    WaitReady2,
    Capturing,
    Captured,
    Recapturing,
    StageClear,
    GameOver,
    Finished,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StageState {
    APPEARANCE,
    NORMAL,
    RUSH,
    CLEARED,
}

pub struct GameInfo {
    pub stage: u16,
    pub left_ship: u32,
    pub game_state: GameState,
    pub count: u32,
    pub stage_state: StageState,
    pub capture_state: CaptureState,
    pub capture_enemy_fi: FormationIndex,
    pub alive_enemy_count: u32,
    pub score_holder: ScoreHolder,
    pub frame_count: u32,
}

impl GameInfo {
    pub fn new(high_score: u32) -> Self {
        let stage = 0;

        GameInfo {
            stage,
            left_ship: DEFAULT_LEFT_SHIP,
            game_state: GameState::StartStage,
            count: 0,
            stage_state: StageState::APPEARANCE,
            capture_state: CaptureState::NoCapture,
            capture_enemy_fi: FormationIndex(0, 0),
            alive_enemy_count: 0,
            score_holder: ScoreHolder::new(high_score),
            frame_count: 0,
        }
    }

    pub fn update(
        &mut self, stage_indicator: &mut StageIndicator, formation: &mut Formation,
        appearance_manager: &mut AppearanceManager, attack_manager: &mut AttackManager,
        eneshot_spawner: &mut EneShotSpawner, star_manager: &mut StarManager, sound_queue: &mut SoundQueue,
        world: &mut SubWorld, commands: &mut CommandBuffer,
    ) {
        self.frame_count = self.frame_count.wrapping_add(1);
        self.check_stage_state(&appearance_manager);

        match self.game_state {
            GameState::StartStage => {
                if self.count == 0 {
                    stage_indicator.set_stage(std::cmp::min(self.stage, 255) + 1);
                }
                if stage_indicator.update() {
                    sound_queue.push_play_se(CH_BOMB, SE_COUNT_STAGE);
                }
                self.count += 1;
                if self.count >= 90 {
                    let captured_fighter = if self.capture_state == CaptureState::Captured {
                        Some(FormationIndex(self.capture_enemy_fi.0, self.capture_enemy_fi.1 - 1))
                    } else {
                        None
                    };
                    self.start_next_stage(self.stage, captured_fighter, formation, appearance_manager, attack_manager, eneshot_spawner);
                    self.game_state = GameState::Playing;
                }
            }
            GameState::Playing => {
                if self.stage_state == StageState::CLEARED {  // TODO: Check enemy-shots.
                    self.game_state = GameState::StageClear;
                    self.count = 0;
                }
            }
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
                    if self.count >= WAIT1 {
                        self.next_player(appearance_manager, attack_manager, world, commands);
                    }
                }
            }
            GameState::WaitReady2 => {
                self.count += 1;
                if self.count >= 60 {
                    for player in <&mut Player>::query().iter_mut(world) {
                        enable_player_shot(player, true);
                    }
                    attack_manager.pause(false);
                    star_manager.set_stop(false);
                    self.game_state = GameState::Playing;
                    self.count = 0;
                }
            }
            GameState::Capturing | GameState::Recapturing => {}
            GameState::Captured => {
                self.count += 1;
            }
            GameState::StageClear => {
                self.count += 1;
                if self.count >= 60 {
                    self.stage = self.stage.saturating_add(1);
                    self.game_state = GameState::StartStage;
                    self.count = 0;
                }
            }
            GameState::GameOver => {
                self.count += 1;
                if self.count >= 35 * 60 / 10 {
                    self.game_state = GameState::Finished;
                }
            }
            GameState::Finished => {}
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
        self.count = 0;
    }

    pub fn capture_completed(&mut self) {
        // Reserve calling `next_player` in next frame.
        self.game_state = GameState::WaitReady;
        self.count = WAIT1 - 1;
    }

    pub fn start_recapturing(&mut self) {
        self.game_state = GameState::Recapturing;
        self.capture_state = CaptureState::Recapturing;
    }

    pub fn end_recapturing(&mut self, dual: bool) {
        //self.stage_manager.pause_attack(false);
        self.capture_state = if dual { CaptureState::Dual } else { CaptureState::NoCapture };
        self.capture_enemy_fi = FormationIndex(0, 0);
        //params.star_manager.set_stop(false);
        self.game_state = GameState::Playing;
    }

    pub fn escape_capturing(&mut self) {
        self.capture_state = CaptureState::NoCapture;
        self.capture_enemy_fi = FormationIndex(0, 0);
        //params.star_manager.set_capturing(false);
        //self.player.escape_capturing();
    }

    pub fn escape_ended(&mut self) {
        //self.stage_manager.pause_attack(false);
        self.game_state = GameState::Playing;
    }

    pub fn captured_fighter_destroyed(&mut self) {
        self.capture_state = CaptureState::NoCapture;
        self.capture_enemy_fi = FormationIndex(0, 0);
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

    pub fn next_player(&mut self, appearance_manager: &mut AppearanceManager, attack_manager: &mut AttackManager, world: &mut SubWorld, commands: &mut CommandBuffer) {
        self.left_ship -= 1;
        if self.left_ship == 0 {
            appearance_manager.pause(true);
            attack_manager.pause(true);
            self.game_state = GameState::GameOver;
            self.count = 0;
        } else {
            for (player, pos, entity) in <(&mut Player, &mut Posture, Entity)>::query().iter_mut(world) {
                restart_player(player, *entity, pos, commands);
                enable_player_shot(player, false);
            }
            self.game_state = GameState::WaitReady2;
            self.count = 0;
        }
    }

    pub fn decrement_alive_enemy(&mut self) {
        self.alive_enemy_count -= 1;
    }

    pub fn is_rush(&self) -> bool {
        self.game_state == GameState::Playing && self.stage_state == StageState::RUSH
    }

    fn start_next_stage(
        &mut self, stage: u16, captured_fighter: Option<FormationIndex>, formation: &mut Formation,
        appearance_manager: &mut AppearanceManager, attack_manager: &mut AttackManager,
        eneshot_spawner: &mut EneShotSpawner,
    ) {
        formation.restart();
        appearance_manager.restart(stage, captured_fighter);
        attack_manager.restart(stage);
        eneshot_spawner.restart();
        self.stage_state = StageState::APPEARANCE;
    }

    fn check_stage_state(&mut self, appearance_manager: &AppearanceManager) {
        if self.stage_state == StageState::APPEARANCE {
            if !appearance_manager.done {
                return;
            }
            self.stage_state = StageState::NORMAL;
        }

        let new_state = match self.alive_enemy_count {
            n if n == 0               => StageState::CLEARED,
            n if n <= RUSH_THRESHOLD  => StageState::RUSH,
            _                         => self.stage_state,
        };
        if new_state != self.stage_state {
            self.stage_state = new_state;
        }
    }
}

//

#[derive(Default)]
pub struct EneShotSpawner {
    queue: Vec<Vec2I>,
    shot_paused_count: u32,
}

impl EneShotSpawner {
    pub fn push(&mut self, pos: &Vec2I) {
        self.queue.push(pos.clone());
    }

    pub fn update(&mut self, game_info: &GameInfo, world: &SubWorld, commands: &mut CommandBuffer) {
        if self.shot_paused_count > 0 {
            self.shot_paused_count -= 1;
        } else {
            self.process_queue(game_info, world, commands);
        }
        self.queue.clear();
    }

    pub fn pause_enemy_shot(&mut self, wait: u32) {
        self.shot_paused_count = wait;
    }

    pub fn restart(&mut self) {
        self.shot_paused_count = 0;
        self.queue.clear();
    }

    fn process_queue(&mut self, game_info: &GameInfo, world: &SubWorld, commands: &mut CommandBuffer) {
        // TODO: Limit maximum.
        let player_pos_opt = <(&Player, &Posture)>::query().iter(world)
            .find(|_| true)
            .map(|(_, posture)| posture.0.clone());
        if let Some(target_pos) = player_pos_opt {
            for pos in self.queue.iter() {
                let d = &target_pos - &pos;
                let angle = atan2_lut(d.y, -d.x);  // 0=down
                let limit = ANGLE * ONE * 30 / 360;
                let angle = clamp(angle, -limit, limit);
                let vel = calc_velocity(angle + ANGLE * ONE / 2, calc_ene_shot_speed(game_info.stage));
                commands.push((
                    EneShot(vel),
                    Posture(*pos, 0),
                    CollRect { offset: Vec2I::new(-1, -4), size: Vec2I::new(1, 8) },
                    SpriteDrawable { sprite_name: "ene_shot", offset: Vec2I::new(-2, -4) },
                ));
            }
        }
    }
}

fn calc_ene_shot_speed(stage: u16) -> i32 {
    const MAX_STAGE: i32 = 64;
    let per = std::cmp::min(stage as i32, MAX_STAGE) * ONE / MAX_STAGE;
    (ENE_SHOT_SPEED2 - ENE_SHOT_SPEED1) * per / ONE + ENE_SHOT_SPEED1
}

//

pub struct SoundQueue {
    queue: Vec<(u32, &'static str)>,
}

impl SoundQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
    }

    pub fn flush<S: SystemTrait>(&mut self, system: &mut S) {
        for (channel, filename) in self.queue.iter() {
            system.play_se(*channel, filename);
        }
        self.queue.clear();
    }

    pub fn push_play_se(&mut self, channel: u32, filename: &'static str) {
        self.queue.push((channel, filename));
    }
}

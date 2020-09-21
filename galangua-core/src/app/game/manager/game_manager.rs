use super::score_holder::ScoreHolder;
use super::enemy_manager::EnemyManager;
use super::event_queue::EventQueue;
use super::{CaptureState, EventType};

use crate::app::consts::*;
use crate::app::game::effect::{Effect, StageIndicator, StarManager};
use crate::app::game::enemy::Accessor as AccessorForEnemy;
use crate::app::game::enemy::{Enemy, FormationIndex};
use crate::app::game::player::Accessor as AccessorForPlayer;
use crate::app::game::player::{MyShot, Player};
use crate::app::util::unsafe_util::peep;
use crate::app::util::Collidable;
use crate::framework::types::Vec2I;
use crate::framework::{RendererTrait, SystemTrait};
use crate::util::math::ONE;
use crate::util::pad::Pad;

const MYSHOT_COUNT: usize = 2;
const MAX_EFFECT_COUNT: usize = 16;
const DEFAULT_LEFT_SHIP: u32 = 3;

#[derive(PartialEq)]
enum GameState {
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

    #[cfg(debug_assertions)]
    EditTraj,
}

pub struct Params<'a> {
    pub star_manager: &'a mut StarManager,
    pub pad: &'a Pad,
    pub score_holder: &'a mut ScoreHolder,
}

pub struct GameManager {
    state: GameState,
    count: u32,
    stage_indicator: StageIndicator,
    player: Player,
    myshots: [Option<MyShot>; MYSHOT_COUNT],
    enemy_manager: EnemyManager,
    effects: [Option<Effect>; MAX_EFFECT_COUNT],
    event_queue: EventQueue,
    stage: u16,
    left_ship: u32,
    capture_state: CaptureState,
    capture_enemy_fi: FormationIndex,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            state: GameState::Playing,
            count: 0,
            stage_indicator: StageIndicator::new(),
            player: Player::new(),
            myshots: Default::default(),
            enemy_manager: EnemyManager::new(),
            event_queue: EventQueue::new(),
            effects: Default::default(),

            stage: 0,
            left_ship: 0,
            capture_state: CaptureState::NoCapture,
            capture_enemy_fi: FormationIndex(0, 0),
        }
    }

    #[cfg(debug_assertions)]
    pub fn enemy_manager_mut(&mut self) -> &mut EnemyManager {
        &mut self.enemy_manager
    }

    pub fn restart(&mut self) {
        self.stage = 0;
        self.stage_indicator.set_stage(self.stage + 1);
        self.left_ship = DEFAULT_LEFT_SHIP;
        self.capture_state = CaptureState::NoCapture;

        self.event_queue.clear();
        self.player = Player::new();

        self.myshots = Default::default();
        self.effects = Default::default();

        self.state = GameState::StartStage;
        self.count = 0;
    }

    #[cfg(debug_assertions)]
    pub fn start_edit_mode(&mut self) {
        self.stage = 0;
        self.stage_indicator.set_stage(self.stage + 1);

        self.enemy_manager.reset_stable();
        self.event_queue.clear();
        self.player = Player::new();

        self.myshots = Default::default();
        self.effects = Default::default();

        self.state = GameState::EditTraj;
    }

    pub fn is_finished(&mut self) -> bool {
        self.state == GameState::Finished
    }

    pub fn update<S: SystemTrait>(&mut self, params: &mut Params, system: &mut S) {
        self.update_common(params, system);

        match self.state {
            GameState::StartStage => {
                self.stage_indicator.update(system);
                self.count += 1;
                if self.count >= 90 {
                    let captured_fighter = if self.capture_state == CaptureState::Captured {
                        Some(FormationIndex(self.capture_enemy_fi.0, self.capture_enemy_fi.1 - 1))
                    } else {
                        None
                    };
                    self.enemy_manager.start_next_stage(self.stage, captured_fighter);
                    self.state = GameState::Playing;
                }
            }
            GameState::Playing => {
                if self.enemy_manager.all_destroyed() {
                    self.state = GameState::StageClear;
                    self.count = 0;
                }
            }
            GameState::Capturing | GameState::Recapturing => {}
            GameState::Captured => {
                self.count += 1;
            }
            GameState::PlayerDead => {
                self.count += 1;
                if self.count >= 60 {
                    self.state = GameState::WaitReady;
                    self.count = 0;
                }
            }
            GameState::WaitReady => {
                if self.enemy_manager.is_no_attacker() {
                    self.count += 1;
                    if self.count >= 60 {
                        self.next_player();
                    }
                }
            }
            GameState::WaitReady2 => {
                self.count += 1;
                if self.count >= 60 {
                    self.player.set_shot_enable(true);
                    self.enemy_manager.pause_attack(false);
                    params.star_manager.set_stop(false);
                    self.state = GameState::Playing;
                    self.count = 0;
                }
            }
            GameState::StageClear => {
                self.count += 1;
                if self.count >= 60 {
                    self.stage = self.stage.saturating_add(1);
                    self.stage_indicator.set_stage(std::cmp::min(self.stage, 255) + 1);

                    self.state = GameState::StartStage;
                    self.count = 0;
                }
            }
            GameState::GameOver => {
                self.count += 1;
                if self.count >= 35 * 60 / 10 {
                    self.state = GameState::Finished;
                }
            }
            GameState::Finished => {}

            #[cfg(debug_assertions)]
            GameState::EditTraj => {}
        }
    }

    fn next_player(&mut self) {
        self.left_ship -= 1;
        if self.left_ship == 0 {
            self.enemy_manager.pause_attack(true);
            self.state = GameState::GameOver;
            self.count = 0;
        } else {
            self.player.restart();
            self.player.set_shot_enable(false);
            self.state = GameState::WaitReady2;
            self.count = 0;
        }
    }

    fn update_common<S: SystemTrait>(&mut self, params: &mut Params, system: &mut S) {
        {
            let accessor = unsafe { peep(self) };
            self.player.update(params.pad, accessor);
            for myshot_opt in self.myshots.iter_mut().filter(|x| x.is_some()) {
                let myshot = myshot_opt.as_mut().unwrap();
                if !myshot.update() {
                    *myshot_opt = None;
                }
            }
        }

        {
            let accessor = unsafe { peep(self) };
            self.enemy_manager.update(accessor);
        }

        // For MyShot.
        self.handle_event_queue(params, system);

        //

        self.check_collision();

        self.handle_event_queue(params, system);

        for effect_opt in self.effects.iter_mut().filter(|x| x.is_some()) {
            let effect = effect_opt.as_mut().unwrap();
            if !effect.update() {
                *effect_opt = None;
            }
        }
    }

    pub fn draw<R: RendererTrait>(&mut self, renderer: &mut R) {
        self.player.draw(renderer);
        self.enemy_manager.draw(renderer);
        for myshot in self.myshots.iter().flat_map(|x| x) {
            myshot.draw(renderer);
        }

        for effect in self.effects.iter().flat_map(|x| x) {
            effect.draw(renderer);
        }
        self.stage_indicator.draw(renderer);

        if self.left_ship > 0 {
            let disp_count = std::cmp::min(self.left_ship - 1, 8);
            for i in 0..disp_count {
                renderer.draw_sprite("rustacean", &Vec2I::new(i as i32 * 16, HEIGHT - 16));
            }
        }

        match self.state {
            GameState::StartStage => {
                renderer.set_texture_color_mod("font", 0, 255, 255);
                renderer.draw_str("font", 10 * 8, 18 * 8, &format!("STAGE {}", self.stage + 1));
            }
            GameState::WaitReady | GameState::WaitReady2 => {
                if self.left_ship > 1 || self.state == GameState::WaitReady2 {
                    renderer.set_texture_color_mod("font", 0, 255, 255);
                    renderer.draw_str("font", (28 - 6) / 2 * 8, 18 * 8, "READY");
                }
            }
            GameState::Captured => {
                if self.count < 120 {
                    renderer.set_texture_color_mod("font", 255, 0, 0);
                    renderer.draw_str("font", (28 - 16) / 2 * 8, 19 * 8, "FIGHTER CAPTURED");
                }
            }
            GameState::GameOver => {
                renderer.set_texture_color_mod("font", 0, 255, 255);
                renderer.draw_str("font", (28 - 8) / 2 * 8, 18 * 8, "GAME OVER");
            }
            _ => {}
        }
    }

    fn handle_event_queue<S: SystemTrait>(&mut self, params: &mut Params, system: &mut S) {
        let mut i = 0;
        while i < self.event_queue.len() {
            match self.event_queue[i] {
                EventType::MyShot(pos, dual, angle) => {
                    if self.spawn_myshot(&pos, dual, angle) {
                        system.play_se(CH_SHOT, SE_MYSHOT);
                    }
                }
                EventType::EneShot(pos) => {
                    self.spawn_ene_shot(&pos);
                }
                EventType::AddScore(add) => {
                    self.add_score(params.score_holder.score, add, system);
                    params.score_holder.add_score(add);
                }
                EventType::EarnPointEffect(point_type, pos) => {
                    self.spawn_effect(Effect::create_earned_point(point_type, &pos));
                }
                EventType::EnemyExplosion(pos, angle, enemy_type) => {
                    self.spawn_effect(Effect::create_flash_enemy(&pos, angle, enemy_type));
                    self.spawn_effect(Effect::create_enemy_explosion(&pos));
                    system.play_se(CH_BOMB, SE_BOMB_ENEMY);
                }
                EventType::PlayerExplosion(pos) => {
                    self.spawn_effect(Effect::create_player_explosion(&pos));
                    system.play_se(CH_BOMB, SE_BOMB_PLAYER);
                }
                EventType::DeadPlayer => {
                    params.star_manager.set_stop(true);
                    if self.state != GameState::Recapturing {
                        self.enemy_manager.pause_attack(true);
                        self.state = GameState::PlayerDead;
                        self.count = 0;
                    }
                }
                EventType::StartCaptureAttack(formation_index) => {
                    self.capture_state = CaptureState::CaptureAttacking;
                    self.capture_enemy_fi = formation_index;
                }
                EventType::EndCaptureAttack => {
                    self.capture_state = CaptureState::NoCapture;
                    self.capture_enemy_fi = FormationIndex(0, 0);
                }
                EventType::CapturePlayer(capture_pos) => {
                    params.star_manager.set_capturing(true);
                    self.enemy_manager.pause_attack(true);
                    self.player.start_capture(&capture_pos);
                    self.state = GameState::Capturing;
                    self.capture_state = CaptureState::Capturing;
                }
                EventType::CapturePlayerCompleted => {
                    params.star_manager.set_capturing(false);
                    self.player.complete_capture();
                    self.capture_state = CaptureState::Captured;
                    self.state = GameState::Captured;
                    self.count = 0;
                }
                EventType::CaptureSequenceEnded => {
                    self.next_player();
                }
                EventType::SpawnCapturedFighter(pos, formation_index) => {
                    self.enemy_manager.spawn_captured_fighter(&pos, &formation_index);
                }
                EventType::RecapturePlayer(captured_fighter_index) => {
                    if let Some(captured_fighter) = self.enemy_manager.get_enemy_at(
                        &captured_fighter_index)
                    {
                        let pos = captured_fighter.pos();
                        self.player.start_recapture_effect(&pos);
                        self.enemy_manager.remove_enemy(&captured_fighter_index);
                        self.enemy_manager.pause_attack(true);
                        self.state = GameState::Recapturing;
                        self.capture_state = CaptureState::Recapturing;
                    }
                }
                EventType::MovePlayerHomePos => {
                    self.player.start_move_home_pos();
                }
                EventType::RecaptureEnded(dual) => {
                    self.enemy_manager.pause_attack(false);
                    self.capture_state = if dual { CaptureState::Dual } else { CaptureState::NoCapture };
                    self.capture_enemy_fi = FormationIndex(0, 0);
                    params.star_manager.set_stop(false);
                    self.state = GameState::Playing;
                }
                EventType::EscapeCapturing => {
                    self.capture_state = CaptureState::NoCapture;
                    self.capture_enemy_fi = FormationIndex(0, 0);
                    params.star_manager.set_capturing(false);
                    self.player.escape_capturing();
                }
                EventType::EscapeEnded => {
                    self.enemy_manager.pause_attack(false);
                    self.state = GameState::Playing;
                }
                EventType::CapturedFighterDestroyed => {
                    self.capture_state = CaptureState::NoCapture;
                    self.capture_enemy_fi = FormationIndex(0, 0);
                }
                EventType::PlaySe(channel, asset_path) => {
                    system.play_se(channel, &asset_path);
                }
            }
            i += 1;
        }
        self.event_queue.clear();
    }

    fn add_score<S: SystemTrait>(&mut self, before: u32, add: u32, system: &mut S) {
        let ext = if before < EXTEND_FIRST_SCORE {
            EXTEND_FIRST_SCORE
        } else {
            (before + EXTEND_AFTER_SCORE - 1) / EXTEND_AFTER_SCORE * EXTEND_AFTER_SCORE
        };
        if before + add >= ext {
            self.extend_ship(system);
        }
    }

    fn extend_ship<S: SystemTrait>(&mut self, system: &mut S) {
        self.left_ship += 1;
        system.play_se(CH_JINGLE, SE_EXTEND_SHIP);
    }

    fn spawn_myshot(&mut self, pos: &Vec2I, dual: bool, angle: i32) -> bool {
        if let Some(myshot_opt) = self.myshots.iter_mut().find(|x| x.is_none()) {
            *myshot_opt = Some(MyShot::new(pos, dual, angle));
            true
        } else {
            false
        }
    }

    fn spawn_ene_shot(&mut self, pos: &Vec2I) {
        let player_pos = [
            Some(*self.player.pos()),
            self.player.dual_pos(),
        ];
        let speed = calc_ene_shot_speed(self.stage);
        self.enemy_manager.spawn_shot(pos, &player_pos, speed);
    }

    fn check_collision(&mut self) {
        #[cfg(debug_assertions)]
        if self.state == GameState::EditTraj {
            return;
        }

        self.check_collision_myshot_enemy();
        self.check_collision_player_enemy();
    }

    fn check_collision_myshot_enemy(&mut self) {
        let power = 1;
        let accessor = unsafe { peep(self) };
        for myshot_opt in self.myshots.iter_mut().filter(|x| x.is_some()) {
            let myshot = myshot_opt.as_ref().unwrap();
            let colls = [
                myshot.get_collbox(),
                myshot.dual_collbox(),
            ];
            let mut hit = false;
            for collbox in colls.iter().flat_map(|x| x) {
                if let Some(fi) = self.enemy_manager.check_collision(collbox) {
                    self.enemy_manager.set_damage_to_enemy(
                        &fi, power, accessor);
                    hit = true;
                }
            }
            if hit {
                *myshot_opt = None;
            }
        }
    }

    fn check_collision_player_enemy(&mut self) {
        let collbox_opts = [
            self.player.dual_collbox().map(|c| (c, self.player.dual_pos().unwrap())),
            self.player.get_collbox().map(|c| (c, *self.player.pos())),
        ];

        for (collbox, player_pos) in collbox_opts.iter().flat_map(|x| x) {
            let power = 100;
            let accessor = unsafe { peep(self) };

            let hit = if let Some(fi) = self.enemy_manager.check_collision(collbox) {
                let pos = self.enemy_manager.get_enemy_at(&fi).unwrap().pos().clone();
                self.enemy_manager.set_damage_to_enemy(&fi, power, accessor);
                Some(pos)
            } else {
                self.enemy_manager.check_shot_collision(&collbox)
            };

            if let Some(pos) = hit {
                self.event_queue.push(EventType::PlayerExplosion(*player_pos));
                if self.player.crash(&pos) {
                    self.event_queue.push(EventType::DeadPlayer);
                } else {
                    // Must be one of dual fighter crashed.
                    assert!(self.capture_state == CaptureState::Dual);
                    self.capture_state = CaptureState::NoCapture;
                }
                continue;
            }
        }
    }

    fn spawn_effect(&mut self, effect: Effect) {
        if let Some(slot) = self.effects.iter_mut().find(|x| x.is_none()) {
            *slot = Some(effect);
        }
    }
}

impl AccessorForPlayer for GameManager {
    fn is_no_attacker(&self) -> bool {
        self.enemy_manager.is_no_attacker()
    }

    fn push_event(&mut self, event: EventType) {
        self.event_queue.push(event);
    }
}

impl AccessorForEnemy for GameManager {
    fn get_player_pos(&self) -> &Vec2I {
        self.player.pos()
    }

    fn get_dual_player_pos(&self) -> Option<Vec2I> {
        self.player.dual_pos()
    }

    fn can_player_capture(&self) -> bool {
        #[cfg(debug_assertions)]
        if self.state == GameState::EditTraj {
            return false;
        }
        self.state == GameState::Playing
    }

    fn is_player_capture_completed(&self) -> bool {
        self.player.is_captured()
    }

    fn capture_state(&self) -> CaptureState {
        self.capture_state
    }

    fn captured_fighter_index(&self) -> Option<FormationIndex> {
        match self.capture_state {
            CaptureState::NoCapture | CaptureState::Recapturing | CaptureState::Dual => {
                None
            }
            _ => {
                Some(FormationIndex(self.capture_enemy_fi.0, self.capture_enemy_fi.1 - 1))
            }
        }
    }

    fn get_enemies(&self) -> &[Option<Box<dyn Enemy>>] {
        self.enemy_manager.get_enemies()
    }

    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Box<dyn Enemy>> {
        self.enemy_manager.get_enemy_at(formation_index)
    }

    fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Box<dyn Enemy>> {
        self.enemy_manager.get_enemy_at_mut(formation_index)
    }

    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.enemy_manager.get_formation_pos(formation_index)
    }

    fn pause_enemy_shot(&mut self, wait: u32) {
        self.enemy_manager.pause_enemy_shot(wait);
    }

    fn is_rush(&self) -> bool {
        self.state == GameState::Playing && self.enemy_manager.is_rush()
    }

    fn get_stage_no(&self) -> u16 {
        self.stage
    }

    fn push_event(&mut self, event: EventType) {
        self.event_queue.push(event);
    }
}

fn calc_ene_shot_speed(stage: u16) -> i32 {
    const MAX_STAGE: i32 = 64;
    let per = std::cmp::min(stage as i32, MAX_STAGE) * ONE / MAX_STAGE;
    (ENE_SHOT_SPEED2 - ENE_SHOT_SPEED1) * per / ONE + ENE_SHOT_SPEED1
}

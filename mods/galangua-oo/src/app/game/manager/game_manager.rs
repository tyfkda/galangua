use super::event_queue::EventQueue;
use super::stage::stage_manager::StageManager;
use super::EventType;

use crate::app::game::effect::Effect;
use crate::app::game::enemy::Accessor as AccessorForEnemy;
use crate::app::game::enemy::Enemy;
use crate::app::game::player::Accessor as AccessorForPlayer;
use crate::app::game::player::{MyShot, Player};

use galangua_common::app::consts::*;
use galangua_common::app::game::effect_table::FLASH_ENEMY_FRAME;
use galangua_common::app::game::stage_indicator::StageIndicator;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::{CaptureState, FormationIndex};
use galangua_common::app::score_holder::ScoreHolder;
use galangua_common::app::util::collision::Collidable;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::{RendererTrait, SystemTrait};
use galangua_common::util::math::ONE;
use galangua_common::util::pad::Pad;
use galangua_common::util::unsafe_util::peep;

const MYSHOT_COUNT: usize = 2;
const MAX_EFFECT_COUNT: usize = 16;

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
    stage_manager: StageManager,
    effects: [Option<Effect>; MAX_EFFECT_COUNT],
    event_queue: EventQueue,
    stage: u16,
    left_ship: u32,
    capture_state: CaptureState,
    capture_enemy_fi: FormationIndex,
}

impl GameManager {
    pub fn new() -> Self {
        let stage = 0;
        let mut stage_indicator = StageIndicator::default();
        stage_indicator.set_stage(stage + 1);

        Self {
            state: GameState::StartStage,
            count: 0,
            stage_indicator,
            player: Player::new(),
            myshots: Default::default(),
            stage_manager: StageManager::new(),
            event_queue: EventQueue::new(),
            effects: Default::default(),

            stage,
            left_ship: DEFAULT_LEFT_SHIP,
            capture_state: CaptureState::NoCapture,
            capture_enemy_fi: FormationIndex(0, 0),
        }
    }

    #[cfg(debug_assertions)]
    pub fn stage_manager_mut(&mut self) -> &mut StageManager {
        &mut self.stage_manager
    }

    #[cfg(debug_assertions)]
    pub fn start_edit_mode(&mut self) {
        self.stage = 0;
        self.stage_indicator.set_stage(self.stage + 1);

        self.stage_manager.reset_stable();
        self.event_queue.clear();
        self.player = Player::new();

        self.myshots = Default::default();
        self.effects = Default::default();

        self.state = GameState::EditTraj;
    }

    pub fn is_finished(&mut self) -> bool {
        self.state == GameState::Finished
    }

    pub fn update(&mut self, params: &mut Params, system: &mut impl SystemTrait) {
        self.update_common(params, system);

        match self.state {
            GameState::StartStage => {
                if self.stage_indicator.update() {
                    system.play_se(CH_BOMB, SE_COUNT_STAGE);
                }
                self.count += 1;
                if self.count >= 90 {
                    let captured_fighter = if self.capture_state == CaptureState::Captured {
                        Some(FormationIndex(self.capture_enemy_fi.0, self.capture_enemy_fi.1 - 1))
                    } else {
                        None
                    };
                    self.stage_manager.start_next_stage(self.stage, captured_fighter);
                    self.state = GameState::Playing;
                }
            }
            GameState::Playing => {
                if self.stage_manager.all_destroyed() {
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
                if self.stage_manager.is_no_attacker() {
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
                    self.stage_manager.pause_attack(false);
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
            self.stage_manager.pause_attack(true);
            self.state = GameState::GameOver;
            self.count = 0;
        } else {
            self.player.restart();
            self.player.set_shot_enable(false);
            self.state = GameState::WaitReady2;
            self.count = 0;
        }
    }

    fn update_common(&mut self, params: &mut Params, system: &mut impl SystemTrait) {
        self.update_player(params);
        self.update_myshots();
        self.update_enemies();
        self.update_effects();

        // For MyShot.
        self.handle_event_queue(params, system);

        //

        self.check_collision();

        self.handle_event_queue(params, system);
    }

    fn update_player(&mut self, params: &mut Params) {
        let accessor = unsafe { peep(self) };
        self.player.update(params.pad, accessor);
    }

    fn update_myshots(&mut self) {
        for myshot_opt in self.myshots.iter_mut().filter(|x| x.is_some()) {
            let myshot = myshot_opt.as_mut().unwrap();
            if !myshot.update() {
                *myshot_opt = None;
            }
        }
    }

    fn update_enemies(&mut self) {
        let accessor = unsafe { peep(self) };
        self.stage_manager.update(accessor);
    }

    fn update_effects(&mut self) {
        for effect_opt in self.effects.iter_mut().filter(|x| x.is_some()) {
            let effect = effect_opt.as_mut().unwrap();
            if !effect.update() {
                *effect_opt = None;
            }
        }
    }

    pub fn draw(&mut self, renderer: &mut impl RendererTrait) {
        self.player.draw(renderer);
        self.stage_manager.draw(renderer);
        for myshot in self.myshots.iter().flatten() {
            myshot.draw(renderer);
        }

        for effect in self.effects.iter().flatten() {
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

    fn do_push_event(&mut self, event: EventType) {
        self.event_queue.push(event);
    }

    fn handle_event_queue(&mut self, params: &mut Params, system: &mut impl SystemTrait) {
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
                    self.spawn_effect(Effect::create_enemy_explosion(&pos, FLASH_ENEMY_FRAME));
                }
                EventType::PlayerExplosion(pos) => {
                    self.spawn_effect(Effect::create_player_explosion(&pos));
                    system.play_se(CH_BOMB, SE_BOMB_PLAYER);
                }
                EventType::DeadPlayer => {
                    params.star_manager.set_stop(true);
                    if self.state != GameState::Recapturing {
                        self.stage_manager.pause_attack(true);
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
                    self.stage_manager.pause_attack(true);
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
                    self.stage_manager.spawn_captured_fighter(&pos, &formation_index);
                }
                EventType::RecapturePlayer(fi, angle) => {
                    if let Some(captured_fighter) = self.stage_manager.get_enemy_at(&fi) {
                        let pos = captured_fighter.pos();
                        self.player.start_recapture_effect(pos, angle);
                        self.stage_manager.remove_enemy(&fi);
                        self.stage_manager.pause_attack(true);
                        system.play_se(CH_JINGLE, SE_RECAPTURE);
                        self.state = GameState::Recapturing;
                        self.capture_state = CaptureState::Recapturing;
                    }
                }
                EventType::MovePlayerHomePos => {
                    self.player.start_move_home_pos();
                }
                EventType::RecaptureEnded(dual) => {
                    self.stage_manager.pause_attack(false);
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
                    self.stage_manager.pause_attack(false);
                    self.state = GameState::Playing;
                }
                EventType::CapturedFighterDestroyed => {
                    self.capture_state = CaptureState::NoCapture;
                    self.capture_enemy_fi = FormationIndex(0, 0);
                }
                EventType::PlaySe(channel, asset_path) => {
                    system.play_se(channel, asset_path);
                }
            }
            i += 1;
        }
        self.event_queue.clear();
    }

    fn add_score(&mut self, before: u32, add: u32, system: &mut impl SystemTrait) {
        let ext = if before < EXTEND_FIRST_SCORE {
            EXTEND_FIRST_SCORE
        } else {
            (before + EXTEND_AFTER_SCORE - 1) / EXTEND_AFTER_SCORE * EXTEND_AFTER_SCORE
        };
        if before + add >= ext {
            self.extend_ship(system);
        }
    }

    fn extend_ship(&mut self, system: &mut impl SystemTrait) {
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
        self.stage_manager.spawn_shot(pos, &player_pos, speed);
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
            for collbox in colls.iter().flatten() {
                if self.stage_manager.check_collision(collbox, power, accessor) {
                    hit = true;
                }
            }
            if hit {
                *myshot_opt = None;
            }
        }
    }

    fn check_collision_player_enemy(&mut self) {
        let power = 100;
        let accessor = unsafe { peep(self) };
        for i in 0..2 {
            let dual = i != 0;
            let collbox = if dual { self.player.dual_collbox() } else { self.player.get_collbox() };
            if let Some(collbox) = collbox {
                let hit = self.stage_manager.check_collision(
                            &collbox, power, accessor) ||
                    self.stage_manager.check_shot_collision(&collbox);

                if hit {
                    let player_pos = if dual { self.player.dual_pos().unwrap() } else { *self.player.pos() };
                    self.event_queue.push(EventType::PlayerExplosion(player_pos));
                    if self.player.crash(dual) {
                        self.event_queue.push(EventType::DeadPlayer);
                        break;
                    } else {
                        // Must be one of dual fighter crashed.
                        assert!(self.capture_state == CaptureState::Dual);
                        self.capture_state = CaptureState::NoCapture;
                    }
                }
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
        self.stage_manager.is_no_attacker()
    }

    fn push_event(&mut self, event: EventType) { self.do_push_event(event); }
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
            CaptureState::CaptureAttacking |
            CaptureState::Capturing |
            CaptureState::Captured =>
            {
                Some(FormationIndex(self.capture_enemy_fi.0, self.capture_enemy_fi.1 - 1))
            }
            _ => None,
        }
    }

    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&dyn Enemy> {
        self.stage_manager.get_enemy_at(formation_index)
    }

    fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Box<dyn Enemy>> {
        self.stage_manager.get_enemy_at_mut(formation_index)
    }

    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.stage_manager.get_formation_pos(formation_index)
    }

    fn pause_enemy_shot(&mut self, wait: u32) {
        self.stage_manager.pause_enemy_shot(wait);
    }

    fn is_rush(&self) -> bool {
        self.state == GameState::Playing && self.stage_manager.is_rush()
    }

    fn get_stage_no(&self) -> u16 {
        self.stage
    }

    fn push_event(&mut self, event: EventType) { self.do_push_event(event); }
}

fn calc_ene_shot_speed(stage: u16) -> i32 {
    const MAX_STAGE: i32 = 64;
    let per = std::cmp::min(stage as i32, MAX_STAGE) * ONE / MAX_STAGE;
    (ENE_SHOT_SPEED2 - ENE_SHOT_SPEED1) * per / ONE + ENE_SHOT_SPEED1
}

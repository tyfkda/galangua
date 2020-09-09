use super::effect::{Effect, StageIndicator, StarManager};
use super::enemy::Accessor as AccessorForEnemy;
use super::enemy::{Enemy, EnemyManager, FormationIndex};
use super::event_queue::{EventQueue, EventType};
use super::player::Accessor as AccessorForPlayer;
use super::player::{MyShot, Player};
use super::score_holder::ScoreHolder;
use super::CaptureState;

use crate::app::consts::*;
use crate::app::util::unsafe_util::peep;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
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
    stage: u32,
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

    /*pub fn score(&self) -> u32 {
        self.score_holder.score
    }

    pub fn high_score(&self) -> u32 {
        self.score_holder.high_score
    }*/

    pub fn update(&mut self, params: &mut Params) {
        self.update_common(params);

        match self.state {
            GameState::StartStage => {
                self.stage_indicator.update();
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
                        self.next_player(params);
                    }
                }
            }
            GameState::StageClear => {
                self.count += 1;
                if self.count >= 60 {
                    self.stage += 1;
                    self.stage_indicator.set_stage(self.stage + 1);

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

    fn next_player(&mut self, params: &mut Params) {
        self.left_ship -= 1;
        if self.left_ship == 0 {
            self.enemy_manager.pause_attack(true);
            self.state = GameState::GameOver;
            self.count = 0;
        } else {
            self.player.restart();
            self.enemy_manager.pause_attack(false);
            params.star_manager.set_stop(false);
            self.state = GameState::Playing;
        }
    }

    fn update_common(&mut self, params: &mut Params) {
        {
            let accessor = unsafe { peep(self) };
            self.player.update(params.pad, accessor, &mut self.event_queue);
            for myshot_opt in self.myshots.iter_mut().filter(|x| x.is_some()) {
                let myshot = myshot_opt.as_mut().unwrap();
                if !myshot.update() {
                    *myshot_opt = None;
                }
            }
        }

        {
            let accessor = unsafe { peep(self) };
            self.enemy_manager.update(accessor, &mut self.event_queue);
        }

        // For MyShot.
        self.handle_event_queue(params);

        //

        self.check_collision();

        self.handle_event_queue(params);

        for effect_opt in self.effects.iter_mut().filter(|x| x.is_some()) {
            let effect = effect_opt.as_mut().unwrap();
            if !effect.update() {
                *effect_opt = None;
            }
        }
    }

    pub fn draw<R>(&mut self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        self.player.draw(renderer)?;
        self.enemy_manager.draw(renderer)?;
        for myshot in self.myshots.iter().flat_map(|x| x) {
            myshot.draw(renderer)?;
        }

        for effect in self.effects.iter().flat_map(|x| x) {
            effect.draw(renderer)?;
        }
        self.stage_indicator.draw(renderer)?;

        if self.left_ship > 0 {
            for i in 0..self.left_ship - 1 {
                renderer.draw_sprite("rustacean", &Vec2I::new(i as i32 * 16, HEIGHT - 16))?;
            }
        }

        match self.state {
            GameState::StartStage => {
                renderer.set_texture_color_mod("font", 0, 255, 255);
                renderer.draw_str("font", 10 * 8, 18 * 8, &format!("STAGE {}", self.stage + 1))?;
            }
            GameState::WaitReady => {
                if self.left_ship > 1 {
                    renderer.set_texture_color_mod("font", 0, 255, 255);
                    renderer.draw_str("font", (28 - 6) / 2 * 8, 18 * 8, "READY")?;
                }
            }
            GameState::Captured => {
                if self.count < 120 {
                    renderer.set_texture_color_mod("font", 255, 0, 0);
                    renderer.draw_str("font", (28 - 16) / 2 * 8, 19 * 8, "FIGHTER CAPTURED")?;
                }
            }
            GameState::GameOver => {
                renderer.set_texture_color_mod("font", 0, 255, 255);
                renderer.draw_str("font", (28 - 8) / 2 * 8, 18 * 8, "GAME OVER")?;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_event_queue(&mut self, params: &mut Params) {
        let mut i = 0;
        while i < self.event_queue.len() {
            match self.event_queue[i] {
                EventType::MyShot(pos, dual, angle) => {
                    self.spawn_myshot(&pos, dual, angle);
                }
                EventType::EneShot(pos, speed) => {
                    self.spawn_ene_shot(&pos, speed);
                }
                EventType::AddScore(add) => {
                    params.score_holder.add_score(add);
                }
                EventType::EarnPoint(point_type, pos) => {
                    self.spawn_effect(Effect::create_earned_point(point_type, &pos));
                }
                EventType::EnemyExplosion(pos) => {
                    self.spawn_effect(Effect::create_enemy_explosion(&pos));
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
                    self.enemy_manager.pause_attack(false);
                    self.state = GameState::Playing;
                    self.next_player(params);
                }
                EventType::SpawnCapturedFighter(pos, formation_index) => {
                    self.enemy_manager.spawn_captured_fighter(&pos, &formation_index);
                }
                EventType::RecapturePlayer(captured_fighter_index) => {
                    if let Some(captured_fighter) = self.enemy_manager.get_enemy_at(
                        &captured_fighter_index)
                    {
                        let pos = captured_fighter.raw_pos();
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
                EventType::RecaptureEnded => {
                    self.enemy_manager.pause_attack(false);
                    self.capture_state = CaptureState::NoCapture;
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
            }
            i += 1;
        }
        self.event_queue.clear();
    }

    fn spawn_myshot(&mut self, pos: &Vec2I, dual: bool, angle: i32) {
        if let Some(myshot_opt) = self.myshots.iter_mut().find(|x| x.is_none()) {
            *myshot_opt = Some(MyShot::new(pos, dual, angle));
        }
    }

    fn spawn_ene_shot(&mut self, pos: &Vec2I, speed: i32) {
        let player_pos = [
            Some(*self.player.get_raw_pos()),
            self.player.dual_pos(),
        ];
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
            let colls: [Option<CollBox>; 2] = [
                myshot.get_collbox(),
                myshot.get_collbox_for_dual(),
            ];
            for collbox in colls.iter().flat_map(|x| x) {
                if let Some(fi) = self.enemy_manager.check_collision(collbox) {
                    self.enemy_manager.set_damage_to_enemy(
                        &fi, power, accessor, &mut self.event_queue);
                    *myshot_opt = None;
                    break;
                }
            }
        }
    }

    fn check_collision_player_enemy(&mut self) {
        if !self.player.active() {
            return;
        }

        let collbox_opts: [Option<(CollBox, Vec2I)>; 2] = [
            self.player.dual_collbox().map(|c| (c, self.player.dual_pos().unwrap())),
            self.player.get_collbox().map(|c| (c, *self.player.get_raw_pos())),
        ];

        for (collbox, player_pos) in collbox_opts.iter().flat_map(|x| x) {
            let power = 100;
            let accessor = unsafe { peep(self) };
            if let Some(fi) = self.enemy_manager.check_collision(collbox) {
                let pos = self.enemy_manager.get_enemy_at(&fi).unwrap().raw_pos().clone();
                self.enemy_manager.set_damage_to_enemy(&fi, power, accessor, &mut self.event_queue);

                self.spawn_effect(Effect::create_player_explosion(player_pos));

                if self.player.crash(&pos) {
                    self.event_queue.push(EventType::DeadPlayer);
                    continue;
                }
            }

            if let Some(pos) = self.enemy_manager.check_shot_collision(&collbox) {
                self.spawn_effect(Effect::create_player_explosion(player_pos));

                if self.player.crash(&pos) {
                    self.event_queue.push(EventType::DeadPlayer);
                    continue;
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
        self.enemy_manager.is_no_attacker()
    }
}

impl AccessorForEnemy for GameManager {
    fn get_raw_player_pos(&self) -> &Vec2I {
        self.player.get_raw_pos()
    }

    fn is_player_dual(&self) -> bool {
        self.player.is_dual()
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
        return self.capture_state
    }

    fn captured_fighter_index(&self) -> Option<FormationIndex> {
        if self.capture_state == CaptureState::NoCapture {
            None
        } else {
            Some(FormationIndex(self.capture_enemy_fi.0, self.capture_enemy_fi.1 - 1))
        }
    }

    fn get_enemies(&self) -> &[Option<Enemy>] {
        self.enemy_manager.get_enemies()
    }

    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Enemy> {
        self.enemy_manager.get_enemy_at(formation_index)
    }

    fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Enemy> {
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
}

use std::cell::RefCell;
use std::rc::Rc;

use super::enemy::Accessor as AccessorForEnemy;
use super::enemy::{Enemy, EnemyManager, FormationIndex};
use super::event_queue::{EventQueue, EventType};
use super::player::{MyShot, Player};

use super::effect::{EarnedPoint, Effect, SmallBomb, StageIndicator, StarManager};
use crate::app::util::unsafe_util::peep;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::pad::Pad;

const MYSHOT_COUNT: usize = 2;
const MAX_EFFECT_COUNT: usize = 16;

#[derive(PartialEq)]
enum GameState {
    Playing,
    PlayerDead,
    Capturing,
    Recapturing,
    StageClear,
    NextStage,
    GameOver,
    Finished,
}

pub struct GameManager {
    state: GameState,
    count: u32,
    star_manager: Rc<RefCell<StarManager>>,
    stage_indicator: StageIndicator,
    player: Player,
    myshots: [Option<MyShot>; MYSHOT_COUNT],
    enemy_manager: EnemyManager,
    effects: [Option<Effect>; MAX_EFFECT_COUNT],
    event_queue: EventQueue,
    stage: u32,
    score: u32,
    high_score: u32,
}

impl GameManager {
    pub fn new(star_manager: Rc<RefCell<StarManager>>) -> Self {
        Self {
            state: GameState::Playing,
            count: 0,
            star_manager,
            stage_indicator: StageIndicator::new(),
            player: Player::new(),
            myshots: Default::default(),
            enemy_manager: EnemyManager::new(),
            event_queue: EventQueue::new(),
            effects: Default::default(),

            stage: 0,
            score: 0,
            high_score: 1000,  //20_000,
        }
    }

    pub fn restart(&mut self) {
        self.stage = 0;
        self.stage_indicator.set_stage(self.stage + 1);

        self.enemy_manager.restart(self.stage);
        self.event_queue.clear();
        self.player = Player::new();

        self.myshots = Default::default();
        self.effects = Default::default();

        self.state = GameState::Playing;
        self.score = 0;
        self.star_manager.borrow_mut().set_stop(false);
    }

    pub fn is_finished(&mut self) -> bool {
        self.state == GameState::Finished
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn high_score(&self) -> u32 {
        self.high_score
    }

    pub fn update(&mut self, pad: &Pad) {
        self.update_common(pad);

        match self.state {
            GameState::Playing => {
                if self.enemy_manager.all_destroyed() {
                    self.state = GameState::StageClear;
                    self.count = 0;
                }
            }
            GameState::Capturing | GameState::Recapturing => {}
            GameState::PlayerDead => {
                // TODO: Wait all enemies back to formation.
                self.count += 1;
                if self.count >= 60 {
                    self.next_player();
                }
            }
            GameState::StageClear => {
                self.count += 1;
                if self.count >= 60 {
                    self.stage += 1;
                    self.stage_indicator.set_stage(self.stage + 1);

                    self.state = GameState::NextStage;
                    self.count = 0;
                }
            }
            GameState::NextStage => {
                self.stage_indicator.update();
                self.count += 1;
                if self.count >= 90 {
                    self.enemy_manager.start_next_stage(self.stage);
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                self.count += 1;
                if self.count >= 2 * 60 {
                    self.state = GameState::Finished;
                }
            }
            GameState::Finished => {}
        }
    }

    fn next_player(&mut self) {
        if self.player.decrement_and_restart() {
            self.enemy_manager.enable_attack(true);
            self.star_manager.borrow_mut().set_stop(false);
            self.state = GameState::Playing;
        } else {
            self.enemy_manager.enable_attack(false);
            self.state = GameState::GameOver;
            self.count = 0;
        }
    }

    fn update_common(&mut self, pad: &Pad) {
        self.player.update(&pad, &mut self.event_queue);
        for myshot_opt in self.myshots.iter_mut().filter(|x| x.is_some()) {
            let myshot = myshot_opt.as_mut().unwrap();
            if !myshot.update() {
                *myshot_opt = None;
            }
        }

        let accessor = unsafe { peep(self) };
        self.enemy_manager.update(accessor, &mut self.event_queue);

        // For MyShot.
        self.handle_event_queue();

        //

        self.check_collision();

        self.handle_event_queue();

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

        if self.state == GameState::NextStage {
            renderer.set_texture_color_mod("font", 0, 255, 255);
            renderer.draw_str("font", 8 * 10, 8 * 17, &format!("STAGE {}", self.stage + 1))?;
        }

        if self.state == GameState::GameOver {
            renderer.set_texture_color_mod("font", 255, 255, 255);
            renderer.draw_str("font", (28 - 10) / 2 * 8, 8 * 10, "GAME OVER")?;
        }

        Ok(())
    }

    fn handle_event_queue(&mut self) {
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
                    self.score += add;
                    if self.score > self.high_score {
                        self.high_score = self.score;
                    }
                }
                EventType::EarnPoint(point_type, pos) => {
                    self.spawn_effect(Effect::EarnedPoint(EarnedPoint::new(point_type, &pos)));
                }
                EventType::SmallBomb(pos) => {
                    self.spawn_effect(Effect::SmallBomb(SmallBomb::new(&pos)));
                }
                EventType::DeadPlayer => {
                    self.star_manager.borrow_mut().set_stop(true);
                    if self.state != GameState::Recapturing {
                        self.enemy_manager.enable_attack(false);
                        self.state = GameState::PlayerDead;
                        self.count = 0;
                    }
                }
                EventType::CapturePlayer(capture_pos) => {
                    self.star_manager.borrow_mut().set_capturing(true);
                    self.enemy_manager.enable_attack(false);
                    self.player.start_capture(&capture_pos);
                    self.state = GameState::Capturing;
                }
                EventType::CapturePlayerCompleted => {
                    self.star_manager.borrow_mut().set_capturing(false);
                    self.player.complete_capture();
                    self.enemy_manager.set_capture_state(true);
                }
                EventType::CaptureSequenceEnded => {
                    self.enemy_manager.enable_attack(true);
                    self.state = GameState::Playing;
                    self.next_player();
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
                        self.enemy_manager.enable_attack(false);
                        self.state = GameState::Recapturing;
                    }
                }
                EventType::MovePlayerHomePos => {
                    self.player.start_move_home_pos();
                }
                EventType::RecaptureEnded => {
                    self.enemy_manager.enable_attack(true);
                    self.enemy_manager.set_capture_state(false);
                    self.star_manager.borrow_mut().set_stop(false);
                    self.state = GameState::Playing;
                }
                EventType::EscapeCapturing => {
                    self.star_manager.borrow_mut().set_capturing(false);
                    self.player.escape_capturing();
                }
                EventType::EscapeEnded => {
                    self.enemy_manager.enable_attack(true);
                }
                EventType::CapturedFighterDestroyed => {
                    self.enemy_manager.set_capture_state(false);
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
                if let Some(_) = self.enemy_manager.check_collision(
                    collbox, power, accessor, &mut self.event_queue)
                {
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
            if let Some(pos) = self.enemy_manager.check_collision(
                collbox, power, accessor, &mut self.event_queue)
            {
                self.event_queue.push(EventType::SmallBomb(*player_pos));

                if self.player.crash(&pos) {
                    self.event_queue.push(EventType::DeadPlayer);
                    continue;
                }
            }

            if let Some(pos) = self.enemy_manager.check_shot_collision(&collbox) {
                self.event_queue.push(EventType::SmallBomb(*player_pos));

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

impl AccessorForEnemy for GameManager {
    fn get_raw_player_pos(&self) -> &Vec2I {
        self.player.get_raw_pos()
    }

    fn is_player_dual(&self) -> bool {
        self.player.is_dual()
    }

    fn is_player_captured(&self) -> bool {
        self.player.is_captured()
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
}

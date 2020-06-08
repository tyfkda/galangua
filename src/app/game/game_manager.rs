use std::cell::RefCell;
use std::rc::Rc;

use crate::app::effect::StageIndicator;
use crate::app::effect::StarManager;
use crate::app::effect::{EarnedPoint, EarnedPointType, Effect, SmallBomb};
use crate::app::enemy::Accessor as AccessorForEnemy;
use crate::app::enemy::{CaptureState, EnemyCollisionResult, EnemyManager};
use crate::app::game::event_queue::{EventQueue, EventType};
use crate::app::player::MyShot;
use crate::app::player::Player;
use crate::app::util::unsafe_util::peep;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::ONE;
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

        for slot in self.myshots.iter_mut() {
            *slot = None;
        }
        for slot in self.effects.iter_mut() {
            *slot = None;
        }

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
        where R: RendererTrait
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
                    self.player.start_capture(&capture_pos);
                    self.state = GameState::Capturing;
                }
                EventType::CapturePlayerCompleted => {
                    self.star_manager.borrow_mut().set_capturing(false);
                    self.player.complete_capture();
                    self.enemy_manager.set_capture_state(true);
                }
                EventType::CaptureSequenceEnded => {
                    self.state = GameState::Playing;
                    self.next_player();
                }
                EventType::RecapturePlayer(pos) => {
                    self.player.start_recapture_effect(&pos);
                    self.enemy_manager.enable_attack(false);
                    self.state = GameState::Recapturing;
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
            Some(self.player.pos()),
            self.player.dual_pos(),
        ];
        self.enemy_manager.spawn_shot(pos, &player_pos, speed);
    }

    fn check_collision(&mut self) {
        self.check_collision_myshot_enemy();
        self.check_collision_player_enemy();
    }

    fn check_collision_myshot_enemy(&mut self) {
        for myshot_opt in self.myshots.iter_mut().filter(|x| x.is_some()) {
            let myshot = myshot_opt.as_ref().unwrap();
            let colls: [Option<CollBox>; 2] = [
                myshot.get_collbox(),
                myshot.get_collbox_for_dual(),
            ];
            for collbox in colls.iter().flat_map(|x| x) {
                if let Some(_) = handle_collision_enemy(
                    &mut self.enemy_manager, &collbox, 1, true, &mut self.event_queue)
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

        let collbox_opts = [
            self.player.dual_collbox(),
            self.player.get_collbox(),
        ];

        for collbox in collbox_opts.iter().flat_map(|x| x) {
            if let Some(pos) = handle_collision_enemy(
                &mut self.enemy_manager, &collbox, 100, false, &mut self.event_queue)
            {
                if self.player.crash(&pos) {
                    self.event_queue.push(EventType::DeadPlayer);
                    continue;
                }
            }

            if let EnemyCollisionResult::Hit{pos, ..} =
                self.enemy_manager.check_shot_collision(&collbox)
            {
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
        self.player.dual_pos().is_some()
    }

    fn is_player_captured(&self) -> bool {
        self.player.is_captured()
    }
}

fn handle_collision_enemy(
    enemy_manager: &mut EnemyManager, collbox: &CollBox, power: u32, effect: bool,
    event_queue: &mut EventQueue) -> Option<Vec2I>
{
    match enemy_manager.check_collision(&collbox, power) {
        EnemyCollisionResult::NoHit => { /* no hit */ }
        EnemyCollisionResult::Hit { pos, destroyed, point, capture_state } => {
            if destroyed {
                if effect {
                    if point > 0 {
                        event_queue.push(EventType::AddScore(point));
                    }

                    let point_type = match point {
                        1600 => Some(EarnedPointType::Point1600),
                        800 => Some(EarnedPointType::Point800),
                        400 => Some(EarnedPointType::Point400),
                        150 => Some(EarnedPointType::Point150),
                        _ => None,
                    };
                    if let Some(point_type) = point_type {
                        event_queue.push(EventType::EarnPoint(point_type, pos));
                    }

                    event_queue.push(EventType::SmallBomb(pos));
                }

                match capture_state {
                    CaptureState::None => {}
                    CaptureState::BeamTracting | CaptureState::BeamClosing => {
                        event_queue.push(EventType::EscapeCapturing);
                    }
                    CaptureState::Capturing => {
                        event_queue.push(EventType::RecapturePlayer(
                            &pos - &Vec2I::new(0, 16 * ONE)));
                    }
                }
            }
            return Some(pos);
        }
    }
    None
}

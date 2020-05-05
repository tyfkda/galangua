use crate::app::effect::StarManager;
use crate::app::effect::{EarnedPoint, EarnedPointType, Effect, SmallBomb};
use crate::app::enemy::Accessor as AccessorForEnemy;
use crate::app::enemy::{EnemyCollisionResult, EnemyManager};
use crate::app::game::event_queue::{EventQueue, EventType};
use crate::app::player::MyShot;
use crate::app::player::Player;
use crate::app::util::{CollBox, Collidable};
use crate::app::util::unsafe_util::peep;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::pad::{Pad, PAD_START};

const MYSHOT_COUNT: usize = 2;
const MAX_EFFECT_COUNT: usize = 16;

#[derive(PartialEq)]
enum GameState {
    Playing,
    PlayerDead,
    StageClear,
    GameOver,
}

pub struct GameManager {
    state: GameState,
    count: u32,
    star_manager: StarManager,
    player: Player,
    myshots: [Option<MyShot>; MYSHOT_COUNT],
    enemy_manager: EnemyManager,
    effects: [Option<Effect>; MAX_EFFECT_COUNT],
    event_queue: EventQueue,
    stage: u32,
    score: u32,
    high_score: u32,
    frame_count: u32,

    paused: bool,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            state: GameState::Playing,
            count: 0,
            star_manager: StarManager::new(),
            player: Player::new(),
            myshots: Default::default(),
            enemy_manager: EnemyManager::new(),
            event_queue: EventQueue::new(),
            effects: Default::default(),

            stage: 0,
            score: 0,
            high_score: 1000,  //20_000,
            frame_count: 0,

            paused: false,
        }
    }

    fn restart(&mut self) {
        self.enemy_manager.restart();
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
        self.frame_count = 0;
    }

    pub fn update(&mut self, pad: &Pad) {
        self.update_common(pad);

        if self.state != GameState::GameOver {
            if pad.is_trigger(PAD_START) {
                self.paused = !self.paused;
            }
        }

        match self.state {
            GameState::Playing => {
                if self.enemy_manager.all_destroyed() {
                    self.state = GameState::StageClear;
                    self.count = 0;
                }
            }
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
                    self.enemy_manager.start_next_stage(self.stage);
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if pad.is_trigger(PAD_START) {
                    self.restart();
                }
            }
        }
    }

    fn next_player(&mut self) {
        if self.player.decrement_and_restart() {
            self.enemy_manager.enable_attack(true);
            self.state = GameState::Playing;
        } else {
            self.enemy_manager.enable_attack(false);
            self.state = GameState::GameOver;
        }
    }

    fn update_common(&mut self, pad: &Pad) {
        if self.paused {
            return;
        }

        self.frame_count += 1;
        self.star_manager.update();

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
        self.star_manager.draw(renderer)?;
        self.player.draw(renderer)?;
        self.enemy_manager.draw(renderer)?;
        for myshot in self.myshots.iter().flat_map(|x| x) {
            myshot.draw(renderer)?;
        }

        for effect in self.effects.iter().flat_map(|x| x) {
            effect.draw(renderer)?;
        }

        renderer.set_texture_color_mod("font", 255, 0, 0);
        if (self.frame_count & 31) < 16 || self.state != GameState::Playing {
            renderer.draw_str("font", 16 * 2, 16 * 0, "1UP")?;
        }
        renderer.draw_str("font", 16 * 9, 16 * 0, "HIGH SCORE")?;
        renderer.set_texture_color_mod("font", 255, 255, 255);
        renderer.draw_str("font", 16 * 0, 16 * 1, &format!("{:6}0", self.score / 10))?;
        renderer.draw_str("font", 16 * 10, 16 * 1, &format!("{:6}0", self.high_score / 10))?;

        if self.state == GameState::GameOver {
            renderer.draw_str("font", (28 - 10) / 2 * 16, 16 * 10, "GAME OVER")?;
        }

        Ok(())
    }

    fn handle_event_queue(&mut self) {
        let mut i = 0;
        while i < self.event_queue.len() {
            let event = &self.event_queue[i];
            match *event {
                EventType::MyShot(pos, dual) => {
                    self.spawn_myshot(pos, dual);
                }
                EventType::EneShot(pos, speed) => {
                    self.spawn_ene_shot(pos, speed);
                }
                EventType::AddScore(add) => {
                    self.score += add;
                    if self.score > self.high_score {
                        self.high_score = self.score;
                    }
                }
                EventType::EarnPoint(point_type, pos) => {
                    self.spawn_effect(Effect::EarnedPoint(EarnedPoint::new(point_type, pos)));
                }
                EventType::SmallBomb(pos) => {
                    self.spawn_effect(Effect::SmallBomb(SmallBomb::new(pos)));
                }
                EventType::DeadPlayer => {
                    self.enemy_manager.enable_attack(false);
                    self.state = GameState::PlayerDead;
                    self.count = 0;
                }
                EventType::CapturePlayer(capture_pos) => {
                    self.player.start_capture(capture_pos);
                }
                EventType::CapturePlayerCompleted => {
                    self.player.complete_capture();
                    self.enemy_manager.set_capture_state(true);
                }
                EventType::CaptureSequenceEnded => {
                    self.next_player();
                }
                EventType::RecapturePlayer(pos) => {
                    self.player.start_recapture_effect(pos);
                    self.enemy_manager.enable_attack(false);
                }
                EventType::MovePlayerHomePos => {
                    self.player.start_move_home_pos();
                }
                EventType::RecaptureEnded => {
                    self.enemy_manager.enable_attack(true);
                    self.enemy_manager.set_capture_state(false);
                }
            }
            i += 1;
        }
        self.event_queue.clear();
    }

    fn spawn_myshot(&mut self, pos: Vec2I, dual: bool) {
        if let Some(myshot_opt) = self.myshots.iter_mut().find(|x| x.is_none()) {
            *myshot_opt = Some(MyShot::new(pos, dual));
        }
    }

    fn spawn_ene_shot(&mut self, pos: Vec2I, speed: i32) {
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
                Some(myshot.get_collbox()),
                myshot.get_collbox_for_dual(),
            ];
            for collbox in colls.iter().flat_map(|x| x) {
                if let Some(_) = handle_collision_enemy(&mut self.enemy_manager, &collbox, 1, true, &mut self.event_queue) {
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
            Some(self.player.get_collbox()),
        ];

        for collbox in collbox_opts.iter().flat_map(|x| x) {
            if let Some(pos) = handle_collision_enemy(&mut self.enemy_manager, &collbox, 100, false, &mut self.event_queue) {
                if self.player.crash(&pos) {
                    self.event_queue.push(EventType::DeadPlayer);
                    continue;
                }
            }

            if let EnemyCollisionResult::Hit{pos, ..} = self.enemy_manager.check_shot_collision(&collbox) {
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
}

fn handle_collision_enemy(
    enemy_manager: &mut EnemyManager, collbox: &CollBox, power: u32, effect: bool,
    event_queue: &mut EventQueue) -> Option<Vec2I>
{
    match enemy_manager.check_collision(&collbox, power) {
        EnemyCollisionResult::NoHit => { /* no hit */ }
        EnemyCollisionResult::Hit { pos, destroyed, point, capturing_player } => {
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

                if let Some(capturing_player) = capturing_player {
                    event_queue.push(EventType::RecapturePlayer(capturing_player.pos));
                }
            }
            return Some(pos);
        }
    }
    None
}

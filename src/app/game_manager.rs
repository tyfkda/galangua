extern crate sdl2;

use rand::Rng;
use sdl2::render::WindowCanvas;

use super::collision::{CollisionResult, Collidable};
use super::draw_util::draw_str;
use super::effect::{Effect, EarnedPoint, EarnedPointType, SmallBomb};
use super::enemy_manager::EnemyManager;
use super::game_event_queue::{GameEventQueue, GameEvent};
use super::myshot::MyShot;
use super::player::Player;
use super::star_manager::StarManager;
use super::super::framework::texture_manager::TextureManager;
use super::super::util::pad::{Pad, PAD_START};
use super::super::util::types::Vec2I;

const MYSHOT_COUNT: usize = 4;
const MAX_EFFECT_COUNT: usize = 16;

#[derive(PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

pub struct GameManager {
    state: GameState,
    star_manager: StarManager,
    player: Player,
    myshots: [Option<MyShot>; MYSHOT_COUNT],
    enemy_manager: EnemyManager,
    effects: [Option<Effect>; MAX_EFFECT_COUNT],
    event_queue: GameEventQueue,
    score: u32,
    high_score: u32,
    frame_count: u32,
}

impl GameManager {
    pub fn new() -> GameManager {
        GameManager {
            state: GameState::Playing,
            star_manager: StarManager::new(),
            player: Player::new(),
            myshots: Default::default(),
            enemy_manager: EnemyManager::new(),
            event_queue: GameEventQueue::new(),
            effects: Default::default(),

            score: 0,
            high_score: 1000,  //20_000,
            frame_count: 0,
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

        if self.state == GameState::GameOver {
            if pad.is_trigger(PAD_START) {
                self.restart();
            }
        }
    }

    fn update_common(&mut self, pad: &Pad) {
        self.frame_count += 1;
        self.star_manager.update();

        self.player.update(&pad, &mut self.event_queue);
        for i in 0..MYSHOT_COUNT {
            if let Some(myshot) = &mut self.myshots[i] {
                if !myshot.update() {
                    self.myshots[i] = None;
                }
            }
        }

        self.enemy_manager.update(&mut self.event_queue);

        // For MyShot.
        self.handle_event_queue();

        //

        self.check_collision();

        self.handle_event_queue();

        for i in 0..MAX_EFFECT_COUNT {
            if let Some(effect) = &mut self.effects[i] {
                if !effect.update() {
                    self.effects[i] = None;
                }
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, texture_manager: &mut TextureManager) -> Result<(), String> {
        if let Some(texture) = texture_manager.get_mut("chr") {
            self.star_manager.draw(canvas, texture)?;

            self.enemy_manager.draw(canvas, texture)?;

            self.player.draw(canvas, texture)?;

            for myshot in self.myshots.iter().flat_map(|x| x) {
                myshot.draw(canvas, texture)?;
            }

            for effect in self.effects.iter().flat_map(|x| x) {
                effect.draw(canvas, texture)?;
            }
        }

        if let Some(font_texture) = texture_manager.get_mut("font") {
            font_texture.set_color_mod(255, 0, 0);
            if (self.frame_count & 31) < 16 || self.state != GameState::Playing {
                draw_str(canvas, &font_texture, 16 * 2, 16 * 0, "1UP")?;
            }
            draw_str(canvas, &font_texture, 16 * 9, 16 * 0, "HIGH SCORE")?;
            font_texture.set_color_mod(255, 255, 255);
            draw_str(canvas, &font_texture, 16 * 0, 16 * 1, &format!("{:6}0", self.score / 10))?;
            draw_str(canvas, &font_texture, 16 * 10, 16 * 1, &format!("{:6}0", self.high_score / 10))?;

            if self.state == GameState::GameOver {
                draw_str(canvas, &font_texture, (28 - 10) / 2 * 16, 16 * 10, "GAME OVER")?;
            }
        }

        Ok(())
    }

    fn handle_event_queue(&mut self) {
        let mut i = 0;
        while i < self.event_queue.len() {
            let event = self.event_queue.get(i);
            match *event {
                GameEvent::MyShot(pos) => {
                    self.spawn_myshot(pos);
                },
                GameEvent::AddScore(add) => {
                    self.score += add;
                    if self.score > self.high_score {
                        self.high_score = self.score;
                    }
                },
                GameEvent::DeadPlayer => {
                    self.state = GameState::GameOver;
                },
            }
            i += 1;
        }
        self.event_queue.clear();
    }

    fn spawn_myshot(&mut self, pos: Vec2I) {
        let max = if self.player.dual() { 4 } else { 2 };
        let count = self.myshots.iter().flat_map(|x| x).count();
        if count >= max {
            return;
        }

        if let Some(myshot_opt) = self.myshots.iter_mut().find(|x| x.is_none()) {
            *myshot_opt = Some(MyShot::new(pos));
        }
    }

    fn check_collision(&mut self) {
        self.check_collision_myshot_enemy();
        self.check_collision_player_enemy();
    }

    fn check_collision_myshot_enemy(&mut self) {
        for myshot_opt in self.myshots.iter_mut() {
            if let Some(myshot) = &myshot_opt {
                match self.enemy_manager.check_collision(&myshot.get_collbox(), 1) {
                    CollisionResult::NoHit => { /* no hit */ },
                    CollisionResult::Hit(pos, destroyed) => {
                        *myshot_opt = None;
                        if destroyed {
                            self.event_queue.add_score(100);

                            let mut rng = rand::thread_rng();
                            let point_type = match rng.gen_range(0, 16) {
                                0 => Some(EarnedPointType::Point1600),
                                1 => Some(EarnedPointType::Point800),
                                2 => Some(EarnedPointType::Point400),
                                3 => Some(EarnedPointType::Point150),
                                _ => None,
                            };
                            if let Some(point_type) = point_type {
                                self.spawn_effect(Effect::EarnedPoint(EarnedPoint::new(point_type, pos)));
                            }

                            self.spawn_effect(Effect::SmallBomb(SmallBomb::new(pos)));
                        }
                        break;
                    },
                }
            }
        }
    }

    fn check_collision_player_enemy(&mut self) {
        if self.player.dead() {
            return;
        }

        let collbox_opts = [
            self.player.dual_collbox(),
            Some(self.player.get_collbox()),
        ];

        for collbox in collbox_opts.iter().flat_map(|x| x) {
            match self.enemy_manager.check_collision(&collbox, 100) {
                CollisionResult::NoHit => { /* no hit */ },
                CollisionResult::Hit(pos, _) => {
                    if self.player.crash(&pos) {
                        self.event_queue.dead_player();
                    }
                },
            }
        }
    }

    fn spawn_effect(&mut self, effect: Effect) {
        if let Some(slot) = self.effects.iter_mut().find(|x| x.is_none()) {
            *slot = Some(effect);
        }
    }
}

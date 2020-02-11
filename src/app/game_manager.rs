extern crate sdl2;

use sdl2::render::WindowCanvas;

use super::collision::CollisionResult;
use super::draw_util::draw_str;
use super::enemy_manager::EnemyManager;
use super::game_event_queue::{GameEventQueue, GameEvent};
use super::myshot::MyShot;
use super::player::Player;
use super::star_manager::StarManager;
use super::super::framework::texture_manager::TextureManager;
use super::super::util::pad::Pad;

const MYSHOT_COUNT: usize = 4;

pub struct GameManager {
    star_manager: StarManager,
    player: Player,
    myshots: [Option<MyShot>; MYSHOT_COUNT],
    enemy_manager: EnemyManager,
    event_queue: GameEventQueue,
    score: u32,
    high_score: u32,
}

impl GameManager {
    pub fn new() -> GameManager {
        let mut enemy_manager = EnemyManager::new();
        enemy_manager.spawn(100, 100);

        GameManager {
            star_manager: StarManager::new(),
            player: Player::new(),
            myshots: Default::default(),
            enemy_manager,
            event_queue: GameEventQueue::new(),

            score: 0,
            high_score: 20_000,
        }
    }

    pub fn update(&mut self, pad: &Pad) {
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
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, texture_manager: &mut TextureManager) -> Result<(), String> {
        if let Some(texture) = texture_manager.get_mut("chr") {
            self.star_manager.draw(canvas, texture)?;

            self.enemy_manager.draw(canvas, texture)?;

            self.player.draw(canvas, texture)?;

            for myshot in self.myshots.iter().flat_map(|x| x) {
                myshot.draw(canvas, texture)?;
            }
        }

        if let Some(font_texture) = texture_manager.get_mut("font") {
            font_texture.set_color_mod(255, 0, 0);
            draw_str(canvas, &font_texture, 16 * 2, 16 * 0, "1UP")?;
            draw_str(canvas, &font_texture, 16 * 9, 16 * 0, "HIGH SCORE")?;
            font_texture.set_color_mod(255, 255, 255);
            draw_str(canvas, &font_texture, 16 * 1, 16 * 1, &format!("{:5}0", self.score / 10))?;
            draw_str(canvas, &font_texture, 16 * 11, 16 * 1, &format!("{:5}0", self.high_score / 10))?;
        }

        Ok(())
    }

    fn handle_event_queue(&mut self) {
        let mut i = 0;
        while i < self.event_queue.queue.len() {
            let event = &self.event_queue.queue[i];
            match *event {
                GameEvent::MyShot(x, y) => {
                    self.spawn_myshot(x, y);
                },
                GameEvent::AddScore(add) => {
                    self.score += add;
                    if self.score > self.high_score {
                        self.high_score = self.score;
                    }
                },
            }
            i += 1;
        }
        self.event_queue.queue.clear();
    }

    fn spawn_myshot(&mut self, x: i32, y: i32) {
        let max = if self.player.dual() { 4 } else { 2 };
        let count = self.myshots.iter().flat_map(|x| x).count();
        if count >= max {
            return;
        }

        if let Some(myshot_opt) = self.myshots.iter_mut().find(|x| x.is_none()) {
            *myshot_opt = Some(MyShot::new(x, y));
        }
    }

    fn check_collision(&mut self) {
        for myshot_opt in self.myshots.iter_mut() {
            if let Some(myshot) = &myshot_opt {
                match self.enemy_manager.check_myshot_collision(&Box::new(myshot)) {
                    CollisionResult::NoHit => { /* no hit */ },
                    _ => {
                        *myshot_opt = None;
                        self.event_queue.add_score(100);
                        break;
                    },
                }
            }
        }
    }
}

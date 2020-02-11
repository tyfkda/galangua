extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollisionResult};
use super::enemy_manager::{EnemyManager};
use super::game_event_queue::{GameEventQueue, GameEvent};
use super::myshot::{MyShot};
use super::player::{Player};
use super::star_manager::{StarManager};
use super::super::framework::{App};
use super::super::framework::texture_manager::TextureManager;
use super::super::util::fps_calc::{FpsCalc};
use super::super::util::pad::{Pad};

const MYSHOT_COUNT: usize = 4;

pub struct GaragaApp {
    pad: Pad,
    fps_calc: FpsCalc,
    texture_manager: TextureManager,

    star_manager: StarManager,
    player: Player,
    myshots: [Option<MyShot>; MYSHOT_COUNT],
    enemy_manager: EnemyManager,
    event_queue: GameEventQueue,
    score: u32,
    high_score: u32,
}

impl GaragaApp {
    pub fn new() -> GaragaApp {
        let mut enemy_manager = EnemyManager::new();
        enemy_manager.spawn(100, 100);

        GaragaApp {
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            texture_manager: TextureManager::new(),

            star_manager: StarManager::new(),
            player: Player::new(),
            myshots: Default::default(),
            enemy_manager,
            event_queue: GameEventQueue::new(),

            score: 0,
            high_score: 20_000,
        }
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

impl App for GaragaApp {
    fn on_key_down(&mut self, keycode: Keycode) {
        self.pad.on_key_down(keycode);
    }

    fn on_key_up(&mut self, keycode: Keycode) {
        self.pad.on_key_up(keycode);
    }

    fn init(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        self.texture_manager.load(canvas, "assets", &vec!["chr.png", "font.png"])?;

        Ok(())
    }

    fn update(&mut self) {
        self.pad.update();
        self.star_manager.update();
        self.player.update(&self.pad, &mut self.event_queue);
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

    fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.clear();

        if let Some(texture) = self.texture_manager.get_mut("chr") {
            self.star_manager.draw(canvas, texture)?;

            self.enemy_manager.draw(canvas, texture)?;

            self.player.draw(canvas, texture)?;

            for myshot in self.myshots.iter().flat_map(|x| x) {
                myshot.draw(canvas, texture)?;
            }
        }

        if let Some(font_texture) = self.texture_manager.get_mut("font") {
            font_texture.set_color_mod(255, 0, 0);
            draw_str(canvas, &font_texture, 16 * 2, 16 * 0, "1UP")?;
            draw_str(canvas, &font_texture, 16 * 9, 16 * 0, "HIGH SCORE")?;
            font_texture.set_color_mod(255, 255, 255);
            draw_str(canvas, &font_texture, 16 * 1, 16 * 1, &format!("{:5}0", self.score / 10))?;
            draw_str(canvas, &font_texture, 16 * 11, 16 * 1, &format!("{:5}0", self.high_score / 10))?;

            font_texture.set_color_mod(128, 128, 128);
            draw_str(canvas, &font_texture, 16 * 23, 0, &format!("FPS{:2}", self.fps_calc.fps()))?;
            font_texture.set_color_mod(255, 255, 255);
        }

        canvas.present();

        self.fps_calc.update();

        Ok(())
    }
}

fn draw_str(canvas: &mut WindowCanvas, texture: &Texture, x: i32, y: i32, text: &str) -> Result<(), String> {
    let w = 16;
    let h = 16;
    let mut x = x;

    for c in text.chars() {
        let u: i32 = ((c as i32) - (' ' as i32)) % 16 * 8;
        let v: i32 = ((c as i32) - (' ' as i32)) / 16 * 8;
        canvas.copy(&texture,
                    Some(Rect::new(u, v, 8, 8)),
                    Some(Rect::new(x, y, w, h)))?;
        x += w as i32;
    }

    Ok(())
}

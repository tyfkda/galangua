extern crate sdl2;

use sdl2::image::{LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::game_event_queue::{GameEventQueue, GameEvent};
use super::myshot::{MyShot};
use super::player::{Player};
use super::super::framework::{App};
use super::super::util::fps_calc::{FpsCalc};
use super::super::util::pad::{Pad};

pub struct GaragaApp {
    pad: Pad,
    fps_calc: FpsCalc,
    texture: Option<Texture>,
    font_texture: Option<Texture>,

    player: Player,
    myshot: Option<MyShot>,
    event_queue: GameEventQueue,
    score: i32,
    high_score: i32,
}

impl GaragaApp {
    pub fn new() -> GaragaApp {
        GaragaApp {
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            texture: None,
            font_texture: None,

            player: Player::new(),
            myshot: None,
            event_queue: GameEventQueue::new(),

            score: 0,
            high_score: 20_000,
        }
    }

    fn handle_event_queue(&mut self) {
        let mut i = 0;
        while i < self.event_queue.queue.len() {
            let event = &self.event_queue.queue[i];
            match event {
                GameEvent::MyShot(x, y) => {
                    if self.myshot.is_none() {
                        self.myshot = Some(MyShot::new(*x, *y));
                    }
                },
            }
            i += 1;
        }
        self.event_queue.queue.clear();
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
        {
            let texture_creator = canvas.texture_creator();
            let texture = texture_creator.load_texture("assets/chr.png")?;
            self.texture = Some(texture);
        }

        {
            let texture_creator = canvas.texture_creator();
            let texture = texture_creator.load_texture("assets/font.png")?;
            self.font_texture = Some(texture);
        }

        Ok(())
    }

    fn update(&mut self) {
        self.player.update(&self.pad, &mut self.event_queue);
        if let Some(myshot) = &mut self.myshot {
            if !myshot.update() {
                self.myshot = None;
                self.score += 100;
            }
        }
        self.handle_event_queue();
    }

    fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.clear();

        if let Some(texture) = &self.texture {
            self.player.draw(canvas, texture)?;
            if let Some(myshot) = &mut self.myshot {
                myshot.draw(canvas, texture)?;
            }
        }

        if let Some(font_texture) = &mut self.font_texture {
            font_texture.set_color_mod(255, 0, 0);
            draw_str(canvas, &font_texture, 16 * 2, 16 * 0, "1UP")?;
            draw_str(canvas, &font_texture, 16 * 9, 16 * 0, "HIGH SCORE")?;
            font_texture.set_color_mod(255, 255, 255);
            draw_str(canvas, &font_texture, 16 * 1, 16 * 1, &format!("{:6}", self.score))?;
            draw_str(canvas, &font_texture, 16 * 11, 16 * 1, &format!("{:6}", self.high_score))?;

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

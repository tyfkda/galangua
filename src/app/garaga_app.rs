extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
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

    player: Player,
    myshot: Option<MyShot>,
    event_queue: GameEventQueue,
}

impl GaragaApp {
    pub fn new() -> GaragaApp {
        GaragaApp {
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            texture: None,

            player: Player::new(),
            myshot: None,
            event_queue: GameEventQueue::new(),
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
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 16 * 2, 16 * 2)
            .map_err(|e| e.to_string())?;
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..16*2 {
                for x in 0..16*2 {
                    let offset = y*pitch + x*3;
                    buffer[offset] = 255;
                    buffer[offset + 1] = 255;
                    buffer[offset + 2] = 255;
                }
            }
        })?;

        self.texture = Some(texture);

        Ok(())
    }

    fn update(&mut self) {
        self.player.update(&self.pad, &mut self.event_queue);
        if let Some(myshot) = &mut self.myshot {
            if !myshot.update() {
                self.myshot = None;
            }
        }
        self.handle_event_queue();
    }

    fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        if let Some(texture) = &self.texture {
            canvas.clear();

            self.player.draw(canvas, texture)?;
            if let Some(myshot) = &mut self.myshot {
                myshot.draw(canvas, texture)?;
            }

            canvas.present();

            if self.fps_calc.update() {
                canvas.window_mut().set_title(&format!("FPS {}", self.fps_calc.fps())).expect("");
            }

            Ok(())
        } else {
            Err("no texture".to_string())
        }
    }
}

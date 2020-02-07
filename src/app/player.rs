extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::game_event_queue::{GameEventQueue};
use super::super::util::pad::{Pad, PAD_L, PAD_R, PAD_A};

pub struct Player {
    x: i32,
    y: i32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            x: 224 / 2,
            y: 288 - 16 - 8,
        }
    }

    pub fn update(&mut self, pad: &Pad, event_queue: &mut GameEventQueue) {
        if pad.is_pressed(PAD_L) {
            self.x -= 2;
            if self.x < 8 {
                self.x = 8;
            }
        }
        if pad.is_pressed(PAD_R) {
            self.x += 2;
            if self.x > 224 - 8 {
                self.x = 224 - 8;
            }
        }
        if pad.is_trigger(PAD_A) {
            event_queue.spawn_myshot(self.x, self.y - 8);
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        canvas.copy(&texture,
                    Some(Rect::new(0, 0, 16, 16)),
                    Some(Rect::new((self.x - 8) * 2, (self.y - 8) * 2, 16 * 2, 16 * 2)))?;

        Ok(())
    }
}

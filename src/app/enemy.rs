extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::game_event_queue::{GameEventQueue};

pub struct Enemy {
    x: i32,
    y: i32,
}

impl Enemy {
    pub fn new(x: i32, y: i32) -> Enemy {
        Enemy {
            x,
            y,
        }
    }

    pub fn update(&mut self, _event_queue: &mut GameEventQueue) {
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        canvas.copy(&texture,
                    Some(Rect::new(0, 0, 16, 16)),
                    Some(Rect::new((self.x - 8) * 2, (self.y - 8) * 2, 16 * 2, 16 * 2)))?;

        Ok(())
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            left: self.x - 8,
            top: self.y - 8,
            width: 16,
            height: 16,
        }
    }
}

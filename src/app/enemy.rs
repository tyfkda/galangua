extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::game_event_queue::GameEventQueue;
use super::super::util::types::Vec2I;

pub struct Enemy {
    pos: Vec2I,
    vel: Vec2I,
}

impl Enemy {
    pub fn new(pos: Vec2I, vel: Vec2I) -> Enemy {
        Enemy {
            pos,
            vel,
        }
    }

    pub fn pos(&self) -> Vec2I {
        self.pos
    }

    pub fn update(&mut self, _event_queue: &mut GameEventQueue) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        canvas.copy(&texture,
                    Some(Rect::new(0, 0, 16, 16)),
                    Some(Rect::new((self.pos.x - 8) * 2, (self.pos.y - 8) * 2, 16 * 2, 16 * 2)))?;

        Ok(())
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: Vec2I::new(self.pos.x - 8, self.pos.y - 8),
            size: Vec2I::new(16, 16),
        }
    }
}

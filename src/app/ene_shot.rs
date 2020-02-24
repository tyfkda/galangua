use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::super::util::types::Vec2I;

pub struct EneShot {
    pub pos: Vec2I,
    pub vel: Vec2I,
}

impl EneShot {
    pub fn new(pos: Vec2I, vel: Vec2I) -> EneShot {
        EneShot {
            pos,
            vel,
        }
    }

    pub fn pos(&self) -> Vec2I {
        Vec2I::new((self.pos.x + 128) / 256, (self.pos.y + 128) / 256)
    }

    pub fn update(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        let pos = self.pos();
        canvas.copy(&texture,
                    Some(Rect::new(16, 8, 8, 8)),
                    Some(Rect::new((pos.x - 4) * 2, (pos.y - 4) * 2, 8 * 2, 8 * 2)))?;

        Ok(())
    }
}

impl Collidable for EneShot {
    fn get_collbox(&self) -> CollBox {
        let pos = self.pos();
        CollBox {
            top_left: Vec2I::new(pos.x - 1, pos.y - 4),
            size: Vec2I::new(1, 8),
        }
    }
}

extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::super::util::types::Vec2I;

pub struct MyShot {
    pos: Vec2I,
}

impl MyShot {
    pub fn new(pos: Vec2I) -> MyShot {
        MyShot {
            pos,
        }
    }

    pub fn update(&mut self) -> bool {
        self.pos.y -= 8;

        self.pos.y >= 0
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        canvas.copy(&texture,
                    Some(Rect::new(16, 0, 8, 8)),
                    Some(Rect::new((self.pos.x - 4) * 2, (self.pos.y - 4) * 2, 8 * 2, 8 * 2)))?;

        Ok(())
    }
}

impl Collidable for MyShot {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: Vec2I::new(self.pos.x - 1, self.pos.y - 4),
            size: Vec2I::new(1, 8),
        }
    }
}

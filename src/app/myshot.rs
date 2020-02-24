extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::super::util::types::Vec2I;

pub struct MyShot {
    pos: Vec2I,
    dual: bool,
}

impl MyShot {
    pub fn new(pos: Vec2I, dual: bool) -> MyShot {
        MyShot {
            pos,
            dual,
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
        if self.dual {
            canvas.copy(&texture,
                        Some(Rect::new(16, 0, 8, 8)),
                        Some(Rect::new((self.pos.x - 4 + 16) * 2, (self.pos.y - 4) * 2, 8 * 2, 8 * 2)))?;
        }

        Ok(())
    }

    pub fn get_collbox_for_dual(&self) -> Option<CollBox> {
        if self.dual {
            Some(CollBox {
                top_left: Vec2I::new(self.pos.x - 1 + 16, self.pos.y - 4),
                size: Vec2I::new(1, 8),
            })
        } else {
            None
        }
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

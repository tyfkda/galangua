extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::super::consts::*;
use super::super::util::{CollBox, Collidable};
use super::super::super::util::types::Vec2I;

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
        self.pos.y -= 8 * ONE;

        self.pos.y >= 0
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        let pos = self.pos();
        canvas.copy(&texture,
                    Some(Rect::new(16, 0, 8, 8)),
                    Some(Rect::new((pos.x - 4) * 2, (pos.y - 4) * 2, 8 * 2, 8 * 2)))?;
        if self.dual {
            canvas.copy(&texture,
                        Some(Rect::new(16, 0, 8, 8)),
                        Some(Rect::new((pos.x - 4 + 16) * 2, (pos.y - 4) * 2, 8 * 2, 8 * 2)))?;
        }

        Ok(())
    }

    pub fn get_collbox_for_dual(&self) -> Option<CollBox> {
        if self.dual {
            let pos = self.pos();
            Some(CollBox {
                top_left: Vec2I::new(pos.x - 1 + 16, pos.y - 4),
                size: Vec2I::new(1, 8),
            })
        } else {
            None
        }
    }

    fn pos(&self) -> Vec2I {
        Vec2I::new((self.pos.x + ONE / 2) / ONE, (self.pos.y + ONE / 2) / ONE)
    }
}

impl Collidable for MyShot {
    fn get_collbox(&self) -> CollBox {
        let pos = self.pos();
        CollBox {
            top_left: Vec2I::new(pos.x - 1, pos.y - 4),
            size: Vec2I::new(1, 8),
        }
    }
}

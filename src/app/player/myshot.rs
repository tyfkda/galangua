use sdl2::rect::Rect;

use super::super::util::{CollBox, Collidable};
use super::super::super::framework::Renderer;
use super::super::super::util::math::{ONE, round_up};
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

    pub fn draw(&self, renderer: &mut dyn Renderer) -> Result<(), String> {
        let pos = self.pos();
        renderer.draw_texture("chr",
                              Some(Rect::new(16, 0, 8, 8)),
                              Some(Rect::new((pos.x - 4) * 2, (pos.y - 4) * 2, 8 * 2, 8 * 2)))?;
        if self.dual {
            renderer.draw_texture("chr",
                                  Some(Rect::new(16, 0, 8, 8)),
                                  Some(Rect::new((pos.x - 4 + 16) * 2, (pos.y - 4) * 2, 8 * 2, 8 * 2)))?;
        }

        Ok(())
    }

    pub fn get_collbox_for_dual(&self) -> Option<CollBox> {
        if self.dual {
            Some(CollBox {
                top_left: self.pos() + Vec2I::new(-1 + 16, -4),
                size: Vec2I::new(1, 8),
            })
        } else {
            None
        }
    }

    fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }
}

impl Collidable for MyShot {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: self.pos() - Vec2I::new(1, 4),
            size: Vec2I::new(1, 8),
        }
    }
}

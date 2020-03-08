use sdl2::rect::Rect;

use super::super::util::{CollBox, Collidable};
use super::super::super::framework::Renderer;
use super::super::super::util::math::round_up;
use super::super::super::util::types::Vec2I;

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
        round_up(&self.pos)
    }

    pub fn update(&mut self) {
        self.pos += self.vel;
    }

    pub fn draw(&self, renderer: &mut Renderer) -> Result<(), String> {
        let pos = self.pos();
        renderer.draw_texture("chr",
                              Some(Rect::new(16, 8, 8, 8)),
                              Some(Rect::new((pos.x - 4) * 2, (pos.y - 4) * 2, 8 * 2, 8 * 2)))?;

        Ok(())
    }
}

impl Collidable for EneShot {
    fn get_collbox(&self) -> CollBox {
        let pos = self.pos();
        CollBox {
            top_left: pos - Vec2I::new(1, 4),
            size: Vec2I::new(1, 8),
        }
    }
}

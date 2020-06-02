use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::round_up;

pub struct EneShot {
    pub pos: Vec2I,
    pub vel: Vec2I,
}

impl EneShot {
    pub fn new(pos: Vec2I, vel: Vec2I) -> Self {
        Self {
            pos,
            vel,
        }
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn raw_pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn update(&mut self) {
        self.pos += self.vel;
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        let pos = self.pos();
        renderer.draw_sprite("ene_shot", &pos + &Vec2I::new(-2, -4))?;

        Ok(())
    }
}

impl Collidable for EneShot {
    fn get_collbox(&self) -> Option<CollBox> {
        let pos = self.pos();
        Some(CollBox {
            top_left: &pos - &Vec2I::new(1, 4),
            size: Vec2I::new(1, 8),
        })
    }
}

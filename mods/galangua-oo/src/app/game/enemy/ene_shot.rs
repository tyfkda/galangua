use galangua_common::app::consts::*;
use galangua_common::app::util::collision::{CollBox, Collidable};
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{round_vec, ONE};

pub struct EneShot {
    pos: Vec2I,
    vel: Vec2I,
}

impl EneShot {
    pub fn new(pos: &Vec2I, vel: &Vec2I) -> Self {
        Self {
            pos: *pos,
            vel: *vel,
        }
    }

    pub fn update(&mut self) -> bool {
        self.pos += &self.vel;
        !out_of_screen(&self.pos)
    }

    pub fn draw(&self, renderer: &mut impl RendererTrait) {
        let pos = round_vec(&self.pos);
        renderer.draw_sprite("ene_shot", &(&pos + &Vec2I::new(-2, -4)));
    }
}

impl Collidable for EneShot {
    fn get_collbox(&self) -> Option<CollBox> {
        Some(CollBox {
            top_left: &round_vec(&self.pos) + &Vec2I::new(-1, -4),
            size: Vec2I::new(1, 8),
        })
    }
}

fn out_of_screen(pos: &Vec2I) -> bool {
    pos.x < -16 * ONE || pos.x > (WIDTH + 16) * ONE ||
        pos.y < -16 * ONE || pos.y > (HEIGHT + 16) * ONE
}

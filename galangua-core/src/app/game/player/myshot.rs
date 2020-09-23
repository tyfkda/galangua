use crate::app::consts::*;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{calc_velocity, quantize_angle, round_vec, ONE};

pub struct MyShot {
    pos: Vec2I,
    dual: bool,
    angle: i32,
}

impl MyShot {
    pub fn new(pos: &Vec2I, dual: bool, angle: i32) -> Self {
        Self {
            pos: *pos,
            dual,
            angle,
        }
    }

    pub fn update(&mut self) -> bool {
        let margin = 4;
        let top = -margin * ONE;
        if self.angle == 0 {
            self.pos.y -= MYSHOT_SPEED;
            self.pos.y > top
        } else {
            let left = -margin * ONE;
            let right = (WIDTH + margin) * ONE;
            let bottom = (HEIGHT + margin) * ONE;
            self.pos += &calc_velocity(self.angle, MYSHOT_SPEED);
            self.pos.y > top && self.pos.x > left && self.pos.x < right && self.pos.y < bottom
        }
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        let pos = round_vec(&self.pos);
        if self.angle == 0 {
            renderer.draw_sprite("myshot", &(&pos + &Vec2I::new(-2, -4)));
            if self.dual {
                renderer.draw_sprite("myshot", &(&pos + &Vec2I::new(-2 + 16, -4)));
            }
        } else {
            assert!(!self.dual);
            renderer.draw_sprite_rot("myshot", &(&pos + &Vec2I::new(-2, -4)),
                                     quantize_angle(self.angle, ANGLE_DIV), None);
        }
    }

    pub fn dual_collbox(&self) -> Option<CollBox> {
        if self.dual {
            Some(CollBox {
                top_left: &round_vec(&self.pos) + &Vec2I::new(-1 + 16, -4),
                size: Vec2I::new(1, 8),
            })
        } else {
            None
        }
    }
}

impl Collidable for MyShot {
    fn get_collbox(&self) -> Option<CollBox> {
        Some(CollBox {
            top_left: &round_vec(&self.pos) - &Vec2I::new(1, 4),
            size: Vec2I::new(1, 8),
        })
    }
}

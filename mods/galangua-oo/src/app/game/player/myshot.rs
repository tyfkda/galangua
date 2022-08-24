use galangua_common::app::consts::*;
use galangua_common::app::util::collision::{CollBox, Collidable};
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{calc_velocity, quantize_angle, round_vec, ONE};

const SPRITE_NAME: &str = "myshot";

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
        if self.angle == 0 {
            self.pos.y -= MYSHOT_SPEED;
        } else {
            self.pos += &calc_velocity(self.angle, MYSHOT_SPEED);
        }
        !out_of_screen(&self.pos)
    }

    pub fn draw(&self, renderer: &mut impl RendererTrait) {
        let pos = round_vec(&self.pos);
        if self.angle == 0 {
            renderer.draw_sprite(SPRITE_NAME, &(&pos + &Vec2I::new(-2, -4)));
            if self.dual {
                renderer.draw_sprite(SPRITE_NAME, &(&pos + &Vec2I::new(-2 + 16, -4)));
            }
        } else {
            assert!(!self.dual);
            renderer.draw_sprite_rot(
                SPRITE_NAME, &(&pos + &Vec2I::new(-2, -4)),
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

fn out_of_screen(pos: &Vec2I) -> bool {
    const MARGIN: i32 = 4;
    const TOP: i32 = -MARGIN * ONE;
    const LEFT: i32 = -MARGIN * ONE;
    const RIGHT: i32 = (WIDTH + MARGIN) * ONE;
    const BOTTOM: i32 = (HEIGHT + MARGIN) * ONE;
    pos.y < TOP || pos.x < LEFT || pos.x > RIGHT || pos.y > BOTTOM
}

impl Collidable for MyShot {
    fn get_collbox(&self) -> Option<CollBox> {
        Some(CollBox {
            top_left: &round_vec(&self.pos) + &Vec2I::new(-1, -4),
            size: Vec2I::new(1, 8),
        })
    }
}

use array_macro::*;
use rand::Rng;

use crate::app::consts;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;

const STAR_COUNT: usize = 128;

pub struct StarManager {
    stars: [Star; STAR_COUNT],
    frame_count: i32,
    capturing: bool,
}

impl StarManager {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let stars = array![|_i|
            Star {
                pos: Vec2I::new(rng.gen_range(0, consts::WIDTH), rng.gen_range(-16, consts::HEIGHT)),
                t: rng.gen_range(0, 64),
                c: rng.gen_range(1, 8),
            }
        ; STAR_COUNT];

        Self {
            stars,
            frame_count: 0,
            capturing: false,
        }
    }

    pub fn update(&mut self) {
        self.frame_count = (self.frame_count + 1) & 63;

        let mut rng = rand::thread_rng();
        let vy = if self.capturing { -2 } else { 1 };
        for star in self.stars.iter_mut() {
            let mut y = star.pos.y + vy;
            if !self.capturing && y >= consts::HEIGHT {
                y = rng.gen_range(-16, -1);
                star.pos.x = rng.gen_range(0, consts::WIDTH);
                star.c = rng.gen_range(1, 8);
                star.t = rng.gen_range(0, 64);
            } else if self.capturing && y < 0 {
                y = consts::HEIGHT + rng.gen_range(1, 16);
                star.pos.x = rng.gen_range(0, consts::WIDTH);
                star.c = rng.gen_range(1, 8);
                star.t = rng.gen_range(0, 64);
            }
            star.pos.y = y;
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        for star in self.stars.iter() {
            if (self.frame_count + star.t) & 31 < 16 {
                continue;
            }

            let col = &COLOR_TABLE[star.c as usize];
            renderer.set_draw_color(col[0], col[1], col[2]);
            renderer.fill_rect(Some([star.pos, Vec2I::new(1, 1)]))?;
        }

        Ok(())
    }

    pub fn set_capturing(&mut self, value: bool) {
        self.capturing = value;
    }
}

struct Star {
    pos: Vec2I,
    t: i32,
    c: u8,
}

const COLOR_TABLE: [[u8; 3]; 8] = [
    [  0,   0,   0],
    [  0,   0, 255],
    [255,   0,   0],
    [255,   0, 255],
    [  0, 255,   0],
    [  0, 255, 255],
    [255, 255,   0],
    [255, 255, 255],
];

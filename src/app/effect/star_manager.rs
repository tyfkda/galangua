use array_macro::*;
use rand::Rng;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::super::consts;
use super::super::super::framework::RendererTrait;
use super::super::super::util::types::Vec2I;

const STAR_COUNT: usize = 128;

pub struct StarManager {
    stars: [Star; STAR_COUNT],
    frame_count: i32,
}

impl StarManager {
    pub fn new() -> StarManager {
        let mut rng = rand::thread_rng();
        let stars = array![|_i|
            Star {
                pos: Vec2I::new(rng.gen_range(0, consts::WIDTH), rng.gen_range(-16, consts::HEIGHT)),
                t: rng.gen_range(0, 64),
                c: rng.gen_range(1, 8),
            }
        ; STAR_COUNT];

        StarManager {
            stars: stars,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        self.frame_count = (self.frame_count + 1) & 63;

        let mut rng = rand::thread_rng();
        for star in self.stars.iter_mut() {
            let mut y = star.pos.y + 1;
            if y >= consts::HEIGHT {
                y = rng.gen_range(-16, -1);
                star.pos.x = rng.gen_range(0, consts::WIDTH);
                star.c = rng.gen_range(1, 8);
                star.t = rng.gen_range(0, 64);
            }
            star.pos.y = y;
        }
    }

    pub fn draw<Renderer>(&self, renderer: &mut Renderer) -> Result<(), String>
        where Renderer: RendererTrait
    {
        for star in self.stars.iter() {
            if (self.frame_count + star.t) & 31 < 16 {
                continue;
            }

            let col = COLOR_TABLE[star.c as usize];
            renderer.set_draw_color(Color::RGB(col[0], col[1], col[2]));
            renderer.fill_rect(Some(Rect::new(star.pos.x, star.pos.y, 1, 1)))?;
        }

        Ok(())
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

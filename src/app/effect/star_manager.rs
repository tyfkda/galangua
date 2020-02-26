extern crate sdl2;

use rand::Rng;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use std::mem::MaybeUninit;

use super::super::super::util::types::Vec2I;

const STAR_COUNT: usize = 128;

pub struct StarManager {
    stars: [Star; STAR_COUNT],
    frame_count: i32,
}

impl StarManager {
    pub fn new() -> StarManager {
        let mut rng = rand::thread_rng();
        let mut stars: [MaybeUninit<Star>; STAR_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };
        for (_i, element) in stars.iter_mut().enumerate() {
            let star = Star {
                pos: Vec2I::new(rng.gen_range(0, 224), rng.gen_range(-16, 288)),
                t: rng.gen_range(0, 64),
                c: rng.gen_range(1, 8),
            };
            *element = MaybeUninit::new(star);
        }
        let stars = unsafe { std::mem::transmute::<_, [Star; STAR_COUNT]>(stars) };

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
            if y >= 288 {
                y = rng.gen_range(-16, -1);
                star.pos.x = rng.gen_range(0, 224);
                star.c = rng.gen_range(1, 8);
                star.t = rng.gen_range(0, 64);
            }
            star.pos.y = y;
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &mut Texture) -> Result<(), String> {
        for star in self.stars.iter() {
            if (self.frame_count + star.t) & 31 < 16 {
                continue;
            }

            let col = COLOR_TABLE[star.c as usize];
            texture.set_color_mod(col[0], col[1], col[2]);
            canvas.copy(&texture,
                        Some(Rect::new(3 * 8, 0, 1, 1)),
                        Some(Rect::new(star.pos.x * 2, star.pos.y * 2, 1 * 2, 1 * 2)))?;
        }
        texture.set_color_mod(255, 255, 255);

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

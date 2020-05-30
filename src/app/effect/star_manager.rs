use array_macro::*;
use rand::Rng;

use crate::app::consts;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{round_up, ONE};

const STAR_COUNT: usize = 128;

#[derive(PartialEq)]
enum State {
    Stop,
    Normal,
    Capturing,
}

pub struct StarManager {
    state: State,
    frame_count: i32,
    scroll_vel: i32,
    stars: [Star; STAR_COUNT],
}

impl StarManager {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let stars = array![|_i|
            Star {
                pos: Vec2I::new(rng.gen_range(0, consts::WIDTH) * ONE, rng.gen_range(-16, consts::HEIGHT) * ONE),
                t: rng.gen_range(0, 64),
                c: rng.gen_range(1, 8),
            }
        ; STAR_COUNT];

        Self {
            state: State::Normal,
            frame_count: 0,
            scroll_vel: 0,
            stars,
        }
    }

    pub fn update(&mut self) {
        self.frame_count = (self.frame_count + 1) & 63;
        if self.state != State::Stop && self.scroll_vel < ONE {
            self.scroll_vel = (self.scroll_vel + ONE / 32).min(ONE);
        }

        let capturing = self.state == State::Capturing;
        let mut rng = rand::thread_rng();
        let vy = if capturing { -2 * ONE } else { self.scroll_vel };
        for star in self.stars.iter_mut() {
            let mut y = star.pos.y + vy;
            if !capturing && y >= consts::HEIGHT * ONE {
                y = rng.gen_range(-16, -1) * ONE;
                star.pos.x = rng.gen_range(0, consts::WIDTH) * ONE;
                star.c = rng.gen_range(1, 8);
                star.t = rng.gen_range(0, 64);
            } else if capturing && y < 0 {
                y = (consts::HEIGHT + rng.gen_range(1, 16)) * ONE;
                star.pos.x = rng.gen_range(0, consts::WIDTH) * ONE;
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
            let pos = round_up(&star.pos);
            renderer.fill_rect(Some([pos, Vec2I::new(1, 1)]))?;
        }

        Ok(())
    }

    pub fn set_capturing(&mut self, value: bool) {
        self.state = if value { State::Capturing } else { State::Normal };
    }

    pub fn set_stop(&mut self, value: bool) {
        self.state = if value { State::Stop } else { State::Normal };
        self.scroll_vel = 0;
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

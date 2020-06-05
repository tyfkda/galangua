use array_macro::*;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::app::consts;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{round_up, ONE};

const STAR_COUNT: usize = 256;

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
                c: choose_random_color(&mut rng),
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
        let vy = if capturing { -3 * ONE } else { self.scroll_vel };
        for star in self.stars.iter_mut() {
            let mut y = star.pos.y + vy;
            let mut warp = false;
            if !capturing && y >= consts::HEIGHT * ONE {
                y = rng.gen_range(-16, -1) * ONE;
                warp = true;
            } else if capturing && y < 0 {
                y = (consts::HEIGHT + rng.gen_range(1, 16)) * ONE;
                warp = true;
            }
            star.pos.y = y;
            if warp {
                star.pos.x = rng.gen_range(0, consts::WIDTH) * ONE;
                star.c = choose_random_color(&mut rng);
                star.t = rng.gen_range(0, 64);
            }
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        for star in self.stars.iter() {
            if (self.frame_count + star.t) & 31 < 16 {
                continue;
            }

            let r =  star.c >> 16;
            let g = (star.c >> 8) & 0xff;
            let b =  star.c       & 0xff;
            renderer.set_draw_color(r as u8, g as u8, b as u8);
            let pos = round_up(&star.pos);
            renderer.fill_rect(Some([&pos, &Vec2I::new(1, 1)]))?;
        }

        Ok(())
    }

    pub fn set_capturing(&mut self, value: bool) {
        self.state = if value { State::Capturing } else { State::Normal };
    }

    pub fn set_stop(&mut self, stop: bool) {
        if stop {
            self.state = State::Stop;
            self.scroll_vel = 0;
        } else {
            self.state = State::Normal;
        };
    }
}

struct Star {
    pos: Vec2I,
    t: i32,
    c: u32,
}

const COLOR_TABLE: [u32; 4] = [0, 71, 151, 222];

fn choose_random_color(rng: &mut ThreadRng) -> u32 {
    let c = rng.gen_range(1, 4 * 4 * 4);
    let r = c % 4;
    let g = (c / 4) % 4;
    let b = c / 16;
    return (COLOR_TABLE[r] << 16) | (COLOR_TABLE[g] << 8) | COLOR_TABLE[b];
}

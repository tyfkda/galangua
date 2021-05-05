use array_macro::*;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use crate::app::consts::*;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{round_vec, ONE};

const STAR_COUNT: usize = 256;
const MAX_SPEED: i32 = ONE;
const REVERSE_SPEED: i32 = -3 * ONE;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Stop,
    Normal,
    Capturing,
}

#[derive(Clone)]
pub struct StarManager {
    state: State,
    frame_count: i32,
    scroll_vel: i32,
    stars: [Star; STAR_COUNT],
}

impl Default for StarManager {
    fn default() -> Self {
        let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
        let stars = array![_i =>
            Star {
                pos: Vec2I::new(rng.gen_range(0, WIDTH) * ONE,
                                rng.gen_range(-16, HEIGHT) * ONE),
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
}

impl StarManager {
    pub fn update(&mut self) {
        self.frame_count = (self.frame_count + 1) & 63;
        if self.state != State::Stop && self.scroll_vel < MAX_SPEED {
            self.scroll_vel = (self.scroll_vel + ONE / 32).min(MAX_SPEED);
        }

        let capturing = self.state == State::Capturing;
        let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
        let vy = if capturing { REVERSE_SPEED } else { self.scroll_vel };
        for star in self.stars.iter_mut() {
            let mut y = star.pos.y + vy;
            let mut warp = false;
            if !capturing && y >= HEIGHT * ONE {
                y = rng.gen_range(-16, -1) * ONE;
                warp = true;
            } else if capturing && y < 0 {
                y = (HEIGHT + rng.gen_range(1, 16)) * ONE;
                warp = true;
            }
            star.pos.y = y;
            if warp {
                star.pos.x = rng.gen_range(0, WIDTH) * ONE;
                star.c = choose_random_color(&mut rng);
                star.t = rng.gen_range(0, 64);
            }
        }
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        for star in self.stars.iter() {
            if (self.frame_count + star.t) & 31 < 16 {
                continue;
            }

            let r =  star.c >> 16;
            let g = (star.c >> 8) & 0xff;
            let b =  star.c       & 0xff;
            renderer.set_draw_color(r as u8, g as u8, b as u8);
            let pos = round_vec(&star.pos);
            renderer.fill_rect(Some([&pos, &Vec2I::new(1, 1)]));
        }
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
        }
    }
}

#[derive(Clone)]
struct Star {
    pos: Vec2I,
    t: i32,
    c: u32,
}

const COLOR_TABLE: [u32; 4] = [0, 71, 151, 222];

fn choose_random_color<T: Rng>(rng: &mut T) -> u32 {
    let c = rng.gen_range(1, 1 << 6);  // 1 for avoid black.
    let r =  c       & 3;
    let g = (c >> 2) & 3;
    let b =  c >> 4;
    (COLOR_TABLE[r] << 16) | (COLOR_TABLE[g] << 8) | COLOR_TABLE[b]
}

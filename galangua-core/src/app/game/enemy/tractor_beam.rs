use galangua_common::app::game::tractor_beam_table::*;
use galangua_common::app::util::hsv;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{round_vec, ONE};

#[derive(Copy, Clone, PartialEq)]
enum State {
    Opening,
    Full,
    Closing,
    Closed,
    Capturing,
}

pub struct TractorBeam {
    pos: Vec2I,
    state: State,
    count: u32,
    color_count: u32,
    size_count: i32,
}

impl TractorBeam {
    pub fn new(pos: &Vec2I) -> Self {
        Self {
            pos: *pos,
            state: State::Opening,
            count: 0,
            color_count: 0,
            size_count: 0,
        }
    }

    pub fn update(&mut self) {
        self.color_count += 1;

        let n = TRACTOR_BEAM_SPRITE_NAMES.len() as i32;
        match self.state {
            State::Opening => {
                self.size_count += ONE / 3;
                if self.size_count >= n * ONE {
                    self.size_count = n * ONE;
                    self.state = State::Full;
                    self.count = 0;
                }
            }
            State::Full => {
                self.count += 1;
                if self.count >= 3 * 60 {
                    self.state = State::Closing;
                }
            }
            State::Closing => {
                if self.size_count > ONE / 3 {
                    self.size_count -= ONE / 3;
                } else {
                    self.size_count = 0;
                    self.state = State::Closed;
                }
            }
            State::Closed => {}
            State::Capturing => {}
        }
    }

    pub fn draw(&self, renderer: &mut dyn RendererTrait) {
        let pos = round_vec(&self.pos);

        let n = (self.size_count / ONE) as usize;
        if n > 0 {
            let hue = self.color_count * 64;
            let pos = &pos + &Vec2I::new(-24, 0);
            for i in 0..n {
                let sprite_name = TRACTOR_BEAM_SPRITE_NAMES[i];
                set_hsv_color(renderer, sprite_name, hue + i as u32 * 160, 255, 255);
                renderer.draw_sprite(sprite_name, &(&pos + &Vec2I::new(0, TRACTOR_BEAM_Y_OFFSET_TABLE[i])));
            }
            renderer.set_sprite_texture_color_mod(TRACTOR_BEAM_SPRITE_NAMES[0], 255, 255, 255);
        }
    }

    pub fn closed(&self) -> bool {
        self.state == State::Closed
    }

    pub fn can_capture(&self, pos: &Vec2I) -> bool {
        const RANGE: i32 = 24 * ONE;
        if self.state == State::Full {
            let dx = pos.x - self.pos.x;
            return dx >= -RANGE && dx <= RANGE;
        }
        false
    }

    pub fn start_capture(&mut self) {
        self.state = State::Capturing;
    }

    pub fn close_capture(&mut self) {
        self.state = State::Closing;
    }
}

fn set_hsv_color(renderer: &mut dyn RendererTrait, sprite_name: &str, h: u32, s: u8, v: u8) {
    let (r, g, b) = hsv(h, s, v);
    renderer.set_sprite_texture_color_mod(sprite_name, r, g, b);
}

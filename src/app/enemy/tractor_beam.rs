use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{round_up, ONE};

const SPRITE_NAMES: [&str; 29] = [
    "beam00", "beam01", "beam02", "beam03", "beam04", "beam05",
    "beam06", "beam07", "beam08", "beam09", "beam10", "beam11",
    "beam12", "beam13", "beam14", "beam15", "beam16", "beam17",
    "beam18", "beam19", "beam20", "beam21", "beam22", "beam23",
    "beam24", "beam25", "beam26", "beam27", "beam28",
];

const Y_OFFSET_TABLE: [i32; 29] = [
     0,  0,  5,  9, 12, 14,
    17, 20, 22, 25, 28, 30,
    33, 36, 40, 41, 44, 47,
    49, 52, 55, 57, 60, 62,
    65, 68, 70, 73, 76,
];

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Opening,
    Full,
    Closing,
    Closed,
}

#[derive(Debug)]
pub struct TractorBeam {
    pos: Vec2I,
    state: State,
    count: u32,
    color_count: u32,
    size_count: i32,
}

impl TractorBeam {
    pub fn new(pos: Vec2I) -> TractorBeam {
        TractorBeam {
            pos,
            state: State::Opening,
            count: 0,
            color_count: 0,
            size_count: 0,
        }
    }

    pub fn update(&mut self) {
        self.color_count += 1;

        let n = SPRITE_NAMES.len() as i32;
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
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let pos = self.pos();

        let n = (self.size_count / ONE) as usize;
        let tex_name = "chr";
        let hue = self.color_count * 64;
        let pos = pos + Vec2I::new(-24, 0);
        for i in 0..n {
            set_hsv_color(renderer, tex_name, hue + i as u32 * 160, 255, 255);
            renderer.draw_sprite(SPRITE_NAMES[i], pos + Vec2I::new(0, Y_OFFSET_TABLE[i]))?;
        }
        renderer.set_texture_color_mod(tex_name, 255, 255, 255);

        Ok(())
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn closed(&self) -> bool {
        self.state == State::Closed
    }
}

fn set_hsv_color<R: RendererTrait>(renderer: &mut R, tex_name: &str, h: u32, s: u8, v: u8) {
    let (r, g, b) = hsv(h, s, v);
    renderer.set_texture_color_mod(tex_name, r, g, b);
}

fn hsv(h: u32, s: u8, v: u8) -> (u8, u8, u8) {
    let h = h % (256 * 6);
    let t = if (h & 256) == 0 { h & 255 } else { 512 - (h & 255) };
    let max = v;
    let min = max - (s as u32 * max as u32 / 255) as u8;
    let d = (max - min) as u32;
    match h / 256 {
        0 => (max, (t * d / 256) as u8 + min, min),
        1 => ((t * d / 256) as u8 + min, max, min),
        2 => (min, max, (t * d / 256) as u8 + min),
        3 => (min, (t * d / 256) as u8 + min, max),
        4 => ((t * d / 256) as u8 + min, min, max),
        5 => (max, min, (t * d / 256) as u8 + min),
        _ => (128, 128, 128),
    }
}

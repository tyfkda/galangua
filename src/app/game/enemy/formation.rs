use lazy_static::lazy_static;

use super::FormationIndex;

use crate::app::consts::*;
use crate::framework::types::Vec2I;
use crate::util::math::ONE;

const X_COUNT: usize = 10;
const Y_COUNT: usize = 6;
const BASE_Y: i32 = 24;

lazy_static! {
    static ref BASE_X_TABLE: [i32; X_COUNT] = {
        let cx = WIDTH / 2;
        let w = 16;

        let mut table: [i32; X_COUNT] = Default::default();
        for j in 0..X_COUNT {
            let x = cx - ((X_COUNT - 1) as i32) * w / 2 + (j as i32) * w;
            table[j] = x * ONE;
        }
        table
    };
    static ref BASE_Y_TABLE: [i32; Y_COUNT] = {
        let h = 16;

        let mut table: [i32; Y_COUNT] = Default::default();
        for i in 0..Y_COUNT {
            let y = BASE_Y + (i as i32) * h;
            table[i] = y * ONE;
        }
        table
    };
}

#[derive(Copy, Clone, PartialEq)]
enum MovingPat {
    Slide,
    Scale,
}

pub struct Formation {
    xtbl: [i32; X_COUNT],
    ytbl: [i32; Y_COUNT],
    moving_pat: MovingPat,
    moving_count: u32,
    done_appearance: bool,
}

impl Formation {
    pub fn new() -> Self {
        let mut formation = Self {
            xtbl: Default::default(),
            ytbl: Default::default(),
            moving_pat: MovingPat::Slide,
            moving_count: 0,
            done_appearance: false,
        };
        formation.restart();
        formation
    }

    pub fn restart(&mut self) {
        self.moving_pat = MovingPat::Slide;
        self.moving_count = 0;
        self.done_appearance = false;

        for j in 0..X_COUNT {
            self.xtbl[j] = BASE_X_TABLE[j];
        }
        for i in 0..Y_COUNT {
            self.ytbl[i] = BASE_Y_TABLE[i];
        }
    }

    pub fn done_appearance(&mut self) {
        self.done_appearance = true;
    }

    pub fn update(&mut self) {
        match self.moving_pat {
            MovingPat::Slide => self.update_formation_slide(),
            MovingPat::Scale => self.update_formation_scale(),
        }
    }

    fn update_formation_slide(&mut self) {
        let t = (self.moving_count as i32 + 64) & 255;
        let dx = ONE / 2;

        for i in 0..X_COUNT {
            if t < 128 {
                self.xtbl[i] += dx;
            } else {
                self.xtbl[i] -= dx;
            }
        }

        self.moving_count += 1;
        if self.done_appearance && (self.moving_count & 127) == 0 {
            self.moving_pat = MovingPat::Scale;
            self.moving_count = 0;
        }
    }

    fn update_formation_scale(&mut self) {
        let t = (self.moving_count as i32) & 255;
        let bx = WIDTH / 2 * ONE;
        let by = (BASE_Y + 16) * ONE;

        for i in 0..X_COUNT {
            let pos_x = BASE_X_TABLE[i];
            let dx = (pos_x - bx) * 2 * 16 / (5 * 16 * ONE / 2);
            if t < 128 {
                self.xtbl[i] += dx;
            } else {
                self.xtbl[i] -= dx;
            }
        }

        for i in 0..Y_COUNT {
            let pos_y = BASE_Y_TABLE[i];
            let dy = (pos_y - by) * 2 * 16 / (5 * 16 * ONE / 2);
            if t < 128 {
                self.ytbl[i] += dy;
            } else {
                self.ytbl[i] -= dy;
            }
        }

        self.moving_count += 1;
    }

    pub fn pos(&self, index: &FormationIndex) -> Vec2I {
        Vec2I::new(self.xtbl[index.0 as usize], self.ytbl[index.1 as usize])
    }

    pub fn is_settle(&self) -> bool {
        self.done_appearance
    }
}

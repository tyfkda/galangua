use array_macro::*;
use lazy_static::lazy_static;

use crate::app::consts::*;

pub const X_COUNT: usize = 10;
pub const Y_COUNT: usize = 6;

pub const BASE_Y: i32 = 24;

lazy_static! {
    pub static ref BASE_X_TABLE: [i32; X_COUNT] = {
        let cx = WIDTH / 2;
        let w = 16;

        array![j =>
            cx - ((X_COUNT - 1) as i32) * w / 2 + (j as i32) * w
        ; X_COUNT]
    };
    pub static ref BASE_Y_TABLE: [i32; Y_COUNT] = {
        let h = 16;

        array![i =>
            BASE_Y + (i as i32) * h
        ; Y_COUNT]
    };
}

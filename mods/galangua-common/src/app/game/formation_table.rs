use array_macro::*;
use once_cell::sync::Lazy;

use crate::app::consts::*;

pub const X_COUNT: usize = 10;
pub const Y_COUNT: usize = 6;

pub const BASE_Y: i32 = 24;

pub static BASE_X_TABLE: Lazy<[i32; X_COUNT]> = Lazy::new(|| {
    let cx = WIDTH / 2;
    let w = 16;

    array![j =>
        cx - ((X_COUNT - 1) as i32) * w / 2 + (j as i32) * w
    ; X_COUNT]
});
pub static BASE_Y_TABLE: Lazy<[i32; Y_COUNT]> = Lazy::new(|| {
    let h = 16;

    array![i =>
        BASE_Y + (i as i32) * h
    ; Y_COUNT]
});

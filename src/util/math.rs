use lazy_static::lazy_static;

use crate::framework::types::Vec2I;

pub const ONE_BIT: i32 = 8;
pub const ONE: i32 = 1 << ONE_BIT;
pub const ANGLE: i32 = 256;

const ANGLE_SIZE: usize = ANGLE as usize;

lazy_static! {
    // Integer sin and cos table, table size:256 = 360 degree, 1.0 = 256
    pub static ref SIN_TABLE: [i32; ANGLE_SIZE] = gen_sin_table(0);
    pub static ref COS_TABLE: [i32; ANGLE_SIZE] = gen_sin_table(ANGLE_SIZE / 4);
}

fn gen_sin_table(phase: usize) -> [i32; ANGLE as usize] {
    let mut table = [0; ANGLE_SIZE];
    for i in 0..ANGLE_SIZE {
        let angle = ((i + phase) as f64) * (2.0 * std::f64::consts::PI / (ANGLE as f64));
        table[i] = ((ONE as f64) * angle.sin()).round() as i32;
    }
    table
}

pub fn diff_angle(target: i32, base: i32) -> i32 {
    let circumference = ANGLE * ONE;
    ((target - base + circumference / 2) & (circumference - 1)) - circumference / 2
}

#[test]
fn test_diff_angle() {
    assert_eq!(100 * ONE, diff_angle(90 * ONE, -10 * ONE));
    assert_eq!(-90 * ONE, diff_angle(10 * ONE, 100 * ONE));
    assert_eq!((ANGLE - 30 - 100) * ONE, diff_angle(-30 * ONE, 100 * ONE));
}

pub fn quantize_angle(angle: i32, div: i32) -> u8 {
    let off = ANGLE / div;
    (((angle + off * (ONE / 2)) / ONE) & (ANGLE - off)) as u8
}

#[test]
fn test_quantize_angle() {
    assert_eq!(0x80, quantize_angle(0x87 * ONE, 16));
    assert_eq!(0x00, quantize_angle(0xfc * ONE, 16));
    assert_eq!(0xe0, quantize_angle(-0x28 * ONE, 16));
}

pub fn clamp<T>(value: T, min: T, max: T) -> T
    where T: Copy + PartialOrd
{
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[test]
fn test_clamp() {
    assert_eq!(1, clamp(-5, 1, 10));
    assert_eq!(5, clamp(5, 1, 10));
    assert_eq!(10, clamp(15, 1, 10));

    assert_eq!(5.0, clamp(5.0, 1.0, 10.0));
}

pub fn round_up_i32(v: i32) -> i32 {
    (v + ONE / 2) >> ONE_BIT
}

pub fn round_up(v: &Vec2I) -> Vec2I {
    Vec2I::new(round_up_i32(v.x), round_up_i32(v.y))
}

#[test]
fn test_round_up() {
    assert_eq!(Vec2I::new(6, 6), round_up(&Vec2I::new(5 * 256 + 128, 6 * 256 + 127)));
    assert_eq!(Vec2I::new(-10, -10), round_up(&Vec2I::new(-11 * 256 + 128, -10 * 256 + 127)));
}

pub fn calc_velocity(angle: i32, speed: i32) -> Vec2I {
    let a: usize = (((angle + ANGLE / 2) & ((ANGLE - 1) * ONE)) / ONE) as usize;
    let cs = COS_TABLE[a];
    let sn = SIN_TABLE[a];
    Vec2I::new(sn * speed / ONE, -cs * speed / ONE)
}

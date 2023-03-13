use lazy_static::lazy_static;
use std::mem::MaybeUninit;
use std::ops::Mul;

use crate::framework::types::Vec2I;

pub const ONE_BIT: i32 = 8;
pub const ONE: i32 = 1 << ONE_BIT;
pub const ANGLE: i32 = 256;

const ANGLE_SIZE: usize = ANGLE as usize;

const ATAN2_TABLE_BIT: u32 = 7;
const ATAN2_N: usize = 1 << ATAN2_TABLE_BIT;
const ATAN2_TABLE_OFFSET: usize = (ATAN2_N / 2 - 1) * (ATAN2_N / 2 - 2) / 2;
const ATAN2_TABLE_LEN: usize = (ATAN2_N - 1) * (ATAN2_N - 2) / 2 - ATAN2_TABLE_OFFSET;
const ATAN2_SCALE: i32 = 8;  // Assume non-diagonal coordinate

lazy_static! {
    // Integer sin and cos table, table size:256 = 360 degree, 1.0 = 256
    pub static ref SIN_TABLE: [i32; ANGLE_SIZE] = gen_sin_table(0);
    pub static ref COS_TABLE: [i32; ANGLE_SIZE] = gen_sin_table(ANGLE_SIZE / 4);
    // Look up table for atan2.
    static ref ATAN2_TABLE: [u8; ATAN2_TABLE_LEN] = gen_atan2_table();
}

fn gen_sin_table(phase: usize) -> [i32; ANGLE as usize] {
    let mut table = [0; ANGLE_SIZE];
    for (i, elem) in table.iter_mut().enumerate() {
        let angle = ((i + phase) as f64) * (2.0 * std::f64::consts::PI / (ANGLE as f64));
        *elem = ((ONE as f64) * angle.sin()).round() as i32;
    }
    table
}

fn gen_atan2_table() -> [u8; ATAN2_TABLE_LEN] {
    let mut table: [MaybeUninit<u8>; ATAN2_TABLE_LEN] =
        unsafe { MaybeUninit::uninit().assume_init() };
    let n = ATAN2_N;

    let mut i = 0;
    for x in (n / 2)..n {
        for y in 1..x {
            let rad = (y as f64).atan2(x as f64);
            let angle = (rad * (((ANGLE * ATAN2_SCALE) as f64) / (2.0 * std::f64::consts::PI)))
                .round() as u16;
            let angle = angle.min(std::u8::MAX as u16);  // Clamp, just in case.
            table[i] = MaybeUninit::new(angle as u8);
            i += 1;
        }
    }

    unsafe { std::mem::transmute::<_, [u8; ATAN2_TABLE_LEN]>(table) }
}

// Find most significant bit.
fn find_msb(x: u32) -> i32 {
    let mut lo: i32 = -1;
    let mut hi: i32 = 32;
    while hi - lo > 1 {
        let m = lo + ((hi - lo) / 2);
        if x >= 1 << m {
            lo = m;
        } else {
            hi = m;
        }
    }
    hi - 1
}

fn normalize_significand(mut x: i32, mut y: i32, bit: u32) -> (i32, i32) {
    assert!(x >= y && y >= 0);
    assert!(bit > 0);

    if x == 0 {
        return (x, y);
    }

    let mut msb = find_msb(x as u32) as u32;
    if msb >= bit {
        if msb > bit {
            // Round up.
            let add = 1 << (msb - bit);
            x += add;
            y += add;
            if x >= 1 << (msb + 1) {
                msb += 1;
            }
        }
        let s = msb - bit;
        (x >> (s + 1), y >> (s + 1))
    } else if msb + 1 < bit {
        (x << (bit - msb - 1), y << (bit - msb - 1))
    } else {
        (x, y)
    }
}

// Return: 360 degrees = ANGLE * ONE
pub fn atan2_lut(mut x: i32, mut y: i32) -> i32 {
    let table = &ATAN2_TABLE;
    let bit = ATAN2_TABLE_BIT;

    let negx = x.is_negative();
    let negy = y.is_negative();
    x = x.abs();
    y = y.abs();

    let swapxy = if x >= y {
        false
    } else {
        std::mem::swap(&mut x, &mut y);
        true
    };

    if x > 0 {
        let (xx, yy) = normalize_significand(x, y, bit);
        x = xx;
        y = yy;
    }

    let mut ang = if y == 0 {
        0
    } else if x > y {
        let index = (((x - 1) * (x - 2) / 2) + y - 1) as usize - ATAN2_TABLE_OFFSET;
        table[index] as i32 * (ONE / ATAN2_SCALE)
    } else {
        ANGLE * ONE / ATAN2_SCALE
    };

    if swapxy { ang = ANGLE * ONE / 4 - ang; }
    if negx   { ang = ANGLE * ONE / 2 - ang; }
    if negy   { ang = -ang; }
    ang
}

pub fn normalize_angle(angle: i32) -> i32 {
    let circumference = ANGLE * ONE;
    let half = circumference / 2;
    ((angle + half) & (circumference - 1)) - half
}

pub fn diff_angle(target: i32, base: i32) -> i32 {
    normalize_angle(target - base)
}

pub fn quantize_angle(angle: i32, div: i32) -> u8 {
    let round = (ANGLE * ONE + div) / (2 * div);
    let a = ((angle + round) & (ANGLE * ONE - 1)) * div / (ANGLE * ONE);
    (a * ANGLE / div) as u8
}

pub fn clamp<T: Copy + PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn square<T: Mul<Output = T> + Copy>(value: T) -> T {
    value * value
}

pub const fn round_up_i32(v: i32) -> i32 {
    (v + ONE / 2) >> ONE_BIT
}

pub fn round_vec(v: &Vec2I) -> Vec2I {
    Vec2I::new(round_up_i32(v.x), round_up_i32(v.y))
}

pub fn calc_velocity(angle: i32, speed: i32) -> Vec2I {
    let a: usize = (((angle + ANGLE / 2) & ((ANGLE - 1) * ONE)) / ONE) as usize;
    let cs = COS_TABLE[a];
    let sn = SIN_TABLE[a];
    Vec2I::new(sn * speed / ONE, -cs * speed / ONE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_msb() {
        assert_eq!(8, find_msb(256));
        assert_eq!(5, find_msb(48));
        assert_eq!(-1, find_msb(0));
    }

    #[test]
    fn test_normalize_significand() {
        assert_eq!((64, 32), normalize_significand(256, 128, 7));
        assert_eq!((64, 32), normalize_significand(257, 129, 7));
        assert_eq!((65, 33), normalize_significand(258, 130, 7));

        assert_eq!((88, 72), normalize_significand(11, 9, 7));

        assert_eq!((113, 85), normalize_significand(113, 85, 7));

        assert_eq!((64, 63), normalize_significand(2043, 2016, 7));
    }

    #[test]
    fn test_normalize_angle() {
        assert_eq!(30 * ONE, normalize_angle(30 * ONE));
        assert_eq!((150 - ANGLE) * ONE, normalize_angle(150 * ONE));
        assert_eq!(-20 * ONE, normalize_angle(-20 * ONE));
        assert_eq!((ANGLE - 190) * ONE, normalize_angle(-190 * ONE));
    }

    #[test]
    fn test_diff_angle() {
        assert_eq!(100 * ONE, diff_angle(90 * ONE, -10 * ONE));
        assert_eq!(-90 * ONE, diff_angle(10 * ONE, 100 * ONE));
        assert_eq!((ANGLE - 30 - 100) * ONE, diff_angle(-30 * ONE, 100 * ONE));
    }

    #[test]
    fn test_quantize_angle() {
        assert_eq!(0x80, quantize_angle(0x87 * ONE, 16));
        assert_eq!(0x00, quantize_angle(0xfc * ONE, 16));
        assert_eq!(0xe0, quantize_angle(-0x28 * ONE, 16));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(1, clamp(-5, 1, 10));
        assert_eq!(5, clamp(5, 1, 10));
        assert_eq!(10, clamp(15, 1, 10));

        assert_eq!(5.0, clamp(5.0, 1.0, 10.0));
    }

    #[test]
    fn test_round_vec() {
        assert_eq!(Vec2I::new(6, 6), round_vec(&Vec2I::new(5 * 256 + 128, 6 * 256 + 127)));
        assert_eq!(Vec2I::new(-10, -10), round_vec(&Vec2I::new(-11 * 256 + 128, -10 * 256 + 127)));
    }
}

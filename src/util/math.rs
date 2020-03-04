use super::types::Vec2I;

lazy_static! {
    // Integer sin and cos table, table size:256 = 360 degree, 1.0 = 256
    pub static ref SIN_TABLE: [i32; 256] = gen_sin_256(0);
    pub static ref COS_TABLE: [i32; 256] = gen_sin_256(256 / 4);
}

fn gen_sin_256(phase: usize) -> [i32; 256] {
    let mut table = [0; 256];
    for i in 0..256 {
        let angle = ((i + phase) as f64) * (std::f64::consts::PI / 128.0);
        table[i] = (256.0 * angle.sin()).round() as i32;
    }
    table
}

pub fn diff_angle(target: i32, base: i32) -> i32 {
    let circumference = 256 * 256;
    ((target - base + circumference / 2) & (circumference - 1)) - circumference / 2
}

#[test]
fn test_diff_angle() {
    assert_eq!(100 * 256, diff_angle(90 * 256, -10 * 256));
    assert_eq!(-90 * 256, diff_angle(10 * 256, 100 * 256));
    assert_eq!((256 - 30 - 100) * 256, diff_angle(-30 * 256, 100 * 256));
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

pub fn calc_velocity(angle: i32, speed: i32) -> Vec2I {
    let a: usize = (((angle + 128) & (255 * 256)) / 256) as usize;
    let cs = COS_TABLE[a];
    let sn = SIN_TABLE[a];
    Vec2I::new(sn * speed / 256, -cs * speed / 256)
}

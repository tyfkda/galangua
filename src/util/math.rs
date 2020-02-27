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

pub fn calc_velocity(angle: i32, speed: i32) -> (i32, i32) {
    let a: usize = (((angle + 128) & (255 * 256)) / 256) as usize;
    let cs = COS_TABLE[a];
    let sn = SIN_TABLE[a];
    (sn * speed / 256, -cs * speed / 256)
}

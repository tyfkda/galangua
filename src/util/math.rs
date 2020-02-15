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

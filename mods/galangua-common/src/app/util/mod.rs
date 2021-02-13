pub mod collision;

pub fn hsv(h: u32, s: u8, v: u8) -> (u8, u8, u8) {
    let h = h % (256 * 6);
    let t = if (h & 256) == 0 { h & 255 } else { 512 - (h & 255) };
    let max = v;
    let min = max - (s as u32 * max as u32 / 255) as u8;
    let d = (max - min) as u32;
    let c = (t * d / 256) as u8 + min;
    match h / 256 {
        0     => (max, c, min),
        1     => (c, max, min),
        2     => (min, max, c),
        3     => (min, c, max),
        4     => (c, min, max),
        /*5 |*/ _ => (max, min, c),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Trimmed {
    pub sprite_source_size: Rect,
    pub source_size: Size,
}

#[derive(Debug)]
pub struct SpriteSheet {
    pub texture: String,
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: Option<Trimmed>,
}

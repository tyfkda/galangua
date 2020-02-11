#[derive(Debug, Copy, Clone)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub fn new(x: T, y: T) -> Vector2D<T> {
        Vector2D {x, y}
    }
}

pub type Vec2I = Vector2D<i32>;

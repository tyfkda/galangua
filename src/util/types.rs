use std::ops::{Add, AddAssign, Sub, Mul, Div};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub fn new(x: T, y: T) -> Vector2D<T> {
        Vector2D {x, y}
    }
}

impl<T> Add for Vector2D<T>
    where T: Add<Output = T>
{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

impl<T> AddAssign for Vector2D<T>
    where T: AddAssign
{
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T> Sub for Vector2D<T>
    where T: Sub<Output = T>
{
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self { x: self.x - other.x, y: self.y - other.y }
    }
}

impl<Scalar> Mul<Scalar> for Vector2D<Scalar>
    where Scalar: Mul<Output = Scalar> + Copy
{
    type Output = Self;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl<Scalar> Div<Scalar> for Vector2D<Scalar>
    where Scalar: Div<Output = Scalar> + Copy
{
    type Output = Self;
    fn div(self, rhs: Scalar) -> Self::Output {
        Self { x: self.x / rhs, y: self.y / rhs }
    }
}

// Vec2I

pub type Vec2I = Vector2D<i32>;

#[test]
fn test_vec2i_ops() {
    assert_eq!(Vec2I::new(4, 6), Vec2I::new(1, 2) + Vec2I::new(3, 4));
    assert_eq!(Vec2I::new(1, 12), Vec2I::new(10, 20) - Vec2I::new(9, 8));
    assert_eq!(Vec2I::new(33, 69), Vec2I::new(11, 23) * 3);
    assert_eq!(Vec2I::new(1, 45), Vec2I::new(123, 4567) / 100);
}

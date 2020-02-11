use super::super::util::types::Vec2I;

// Collision Result
pub enum CollisionResult {
    NoHit,
    Hit,
    Destroy,
}

// Collision Box
pub struct CollBox {
    pub top_left: Vec2I,
    pub size: Vec2I,
}

impl CollBox {
    pub fn check_collision(&self, target: &CollBox) -> bool {
        let r1 = self.top_left.x + self.size.x;
        let b1 = self.top_left.y + self.size.y;
        let r2 = target.top_left.x + target.size.x;
        let b2 = target.top_left.y + target.size.y;

        self.top_left.x < r2 && self.top_left.y < b2 &&
            target.top_left.x < r1 && target.top_left.y < b1
    }
}

// Collidable
pub trait Collidable {
    fn get_collbox(&self) -> CollBox;

    fn collide_with(&self, target: &Box<&dyn Collidable>) -> CollisionResult {
        let collbox1 = self.get_collbox();
        let collbox2 = target.get_collbox();
        if collbox1.check_collision(&collbox2) {
            CollisionResult::Hit
        } else {
            CollisionResult::NoHit
        }
    }
}

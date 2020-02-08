
// Collision Result
pub enum CollisionResult {
    NoHit,
    Hit,
    Destroy,
}

// Collision Box
pub struct CollBox {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl CollBox {
    pub fn check_collision(&self, target: &CollBox) -> bool {
        let r1 = self.left + self.width;
        let b1 = self.top + self.height;
        let r2 = target.left + target.width;
        let b2 = target.top + target.height;

        self.left < r2 && self.top < b2 &&
            target.left < r1 && target.top < b1
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

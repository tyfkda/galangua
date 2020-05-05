use crate::framework::types::Vec2I;

// Collision Box
pub struct CollBox {
    pub top_left: Vec2I,
    pub size: Vec2I,
}

impl CollBox {
    pub fn check_collision(&self, target: &CollBox) -> bool {
        let br1 = &self.top_left + &self.size;
        let br2 = &target.top_left + &target.size;

        self.top_left.x < br2.x && self.top_left.y < br2.y &&
            target.top_left.x < br1.x && target.top_left.y < br1.y
    }
}

#[test]
fn test_collbox_check_collision() {
    let collbox1 = CollBox { top_left: Vec2I::new(10, 5), size: Vec2I::new(4, 6) };

    let edge = CollBox { top_left: Vec2I::new(14, 7), size: Vec2I::new(3, 2) };
    assert_eq!(false, collbox1.check_collision(&edge));
    assert_eq!(false, edge.check_collision(&collbox1));

    let corner = CollBox { top_left: Vec2I::new(5, 11), size: Vec2I::new(5, 7) };
    assert_eq!(false, collbox1.check_collision(&corner));
    assert_eq!(false, corner.check_collision(&collbox1));

    let inside = CollBox { top_left: Vec2I::new(11, 6), size: Vec2I::new(2, 3) };
    assert_eq!(true, collbox1.check_collision(&inside));
    assert_eq!(true, inside.check_collision(&collbox1));
}

// Collidable
pub trait Collidable {
    fn get_collbox(&self) -> CollBox;

    fn collide_with<T: Collidable>(&self, target: &Box<&T>) -> bool {
        let collbox1 = self.get_collbox();
        let collbox2 = target.get_collbox();
        collbox1.check_collision(&collbox2)
    }
}

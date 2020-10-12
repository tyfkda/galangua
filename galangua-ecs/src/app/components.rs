use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::framework::types::Vec2I;

//
pub struct Pos(pub Vec2I);

//
pub struct CollRect {
    pub offset: Vec2I,
    pub size: Vec2I,
}

//
pub struct Player;

//
pub struct MyShot;

//
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub formation_index: FormationIndex,
}

//
pub struct SpriteDrawable { pub sprite_name: &'static str, pub offset: Vec2I }

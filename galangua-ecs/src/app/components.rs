use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::game::traj::Traj;
use galangua_common::framework::types::Vec2I;

//
#[derive(Clone)]
pub struct Posture(pub Vec2I, pub i32);

//
pub struct Speed(pub i32, pub i32);

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
#[derive(PartialEq)]
pub enum ZakoState {
    Appearance,
    MoveToFormation,
    Formation,
    Attack,
}
pub struct Zako {
    pub state: ZakoState,
    pub traj: Option<Traj>,
}

//
pub struct SpriteDrawable { pub sprite_name: &'static str, pub offset: Vec2I }

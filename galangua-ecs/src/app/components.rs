use specs::{prelude::*, Component};

use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::game::traj::Traj;
use galangua_common::framework::types::Vec2I;

//
#[derive(Clone, Component)]
#[storage(VecStorage)]
pub struct Posture(pub Vec2I, pub i32);

//
#[derive(Clone, Component)]
#[storage(VecStorage)]
pub struct Speed(pub i32, pub i32);

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct CollRect {
    pub offset: Vec2I,
    pub size: Vec2I,
}

//
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Player;

//
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct MyShot;

//
#[derive(Component)]
#[storage(VecStorage)]
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
#[derive(Component)]
#[storage(VecStorage)]
pub struct Zako {
    pub state: ZakoState,
    pub traj: Option<Traj>,
}

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct SequentialSpriteAnime(pub &'static [&'static str], pub u32, pub u32);

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct SpriteDrawable { pub sprite_name: &'static str, pub offset: Vec2I }

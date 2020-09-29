use specs::{prelude::*, Component};

use galangua_common::app::game::EnemyType;
use galangua_common::framework::types::Vec2I;

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct Pos(pub Vec2I);

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
}

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct SpriteDrawable { pub sprite_name: &'static str, pub offset: Vec2I }

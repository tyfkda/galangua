use specs::{prelude::*, Component};

use galangua_common::framework::types::Vec2I;

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct Vel(pub Vec2I);

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct Pos(pub Vec2I);

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct SpriteDrawable;

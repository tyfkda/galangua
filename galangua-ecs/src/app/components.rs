use specs::{prelude::*, Component};

use galangua_common::framework::types::Vec2I;

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct Pos(pub Vec2I);

//
#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Player;

//
#[derive(Component)]
#[storage(VecStorage)]
pub struct SpriteDrawable { pub sprite_name: &'static str, pub offset: Vec2I }

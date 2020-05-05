// Accessor of game information for Enemy.

use crate::framework::types::Vec2I;

pub trait Accessor {
    fn get_raw_player_pos(&self) -> &Vec2I;
    fn is_player_dual(&self) -> bool;
}

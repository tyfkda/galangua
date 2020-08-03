// Accessor of game information for Enemy.

use super::{Enemy, FormationIndex};

use crate::framework::types::Vec2I;

pub trait Accessor {
    fn get_raw_player_pos(&self) -> &Vec2I;
    fn is_player_dual(&self) -> bool;
    fn is_player_captured(&self) -> bool;
    fn can_player_capture(&self) -> bool;
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Enemy>;
    fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Enemy>;
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I;
}

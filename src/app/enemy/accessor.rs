// Accessor of game information for Enemy.

use crate::app::enemy::FormationIndex;
use crate::framework::types::Vec2I;

pub trait Accessor {
    fn get_raw_player_pos(&self) -> &Vec2I;
    fn is_player_dual(&self) -> bool;
    fn is_player_captured(&self) -> bool;
    fn is_enemy_formation(&self, formation_index: &FormationIndex) -> bool;
    fn set_to_troop(&mut self, formation_index: &FormationIndex);
    fn set_to_formation(&mut self, formation_index: &FormationIndex);
    fn update_troop(&mut self, formation_index: &FormationIndex, add: &Vec2I, angle: i32) -> bool;
}

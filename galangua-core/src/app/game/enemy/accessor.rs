// Accessor of game information for Enemy.

use super::{Enemy, FormationIndex};

use crate::app::game::CaptureState;
use crate::framework::types::Vec2I;

pub trait Accessor {
    fn get_raw_player_pos(&self) -> &Vec2I;
    fn is_player_dual(&self) -> bool;
    fn can_player_capture(&self) -> bool;
    fn is_player_capture_completed(&self) -> bool;
    fn capture_state(&self) -> CaptureState;
    fn get_enemies(&self) -> &[Option<Enemy>];
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Enemy>;
    fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Enemy>;
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I;
    fn is_rush(&self) -> bool;
}

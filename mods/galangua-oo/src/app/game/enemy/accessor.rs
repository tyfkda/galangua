// Accessor of game information for Enemy.

use super::Enemy;

use crate::app::game::manager::EventType;

use galangua_common::app::game::{CaptureState, FormationIndex};
use galangua_common::framework::types::Vec2I;

pub trait Accessor {
    fn get_player_pos(&self) -> &Vec2I;
    fn get_dual_player_pos(&self) -> Option<Vec2I>;
    fn can_player_capture(&self) -> bool;
    fn is_player_capture_completed(&self) -> bool;
    fn capture_state(&self) -> CaptureState;
    fn captured_fighter_index(&self) -> Option<FormationIndex>;
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&dyn Enemy>;
    fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex)
        -> Option<&mut Box<dyn Enemy>>;
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I;
    fn pause_enemy_shot(&mut self, wait: u32);
    fn is_rush(&self) -> bool;
    fn get_stage_no(&self) -> u16;

    fn push_event(&mut self, event: EventType);
}

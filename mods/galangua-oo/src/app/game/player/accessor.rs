// Accessor of game information for Player.

use crate::app::game::manager::EventType;

use galangua_common::framework::types::Vec2I;

pub trait Accessor {
    fn spawn_myshot(&mut self, pos: &Vec2I, dual: bool, angle: i32);
    fn is_no_attacker(&self) -> bool;

    fn push_event(&mut self, event: EventType);
}

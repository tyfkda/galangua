// Accessor of game information for Player.

use crate::app::game::manager::CaptureEventType;

use galangua_common::framework::types::Vec2I;

pub trait Accessor {
    fn spawn_myshot(&mut self, pos: &Vec2I, dual: bool, angle: i32);
    fn is_no_attacker(&self) -> bool;

    fn capture_event(&mut self, event: CaptureEventType);
}

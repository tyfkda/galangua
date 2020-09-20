// Accessor of game information for Player.

use crate::app::game::manager::EventType;

pub trait Accessor {
    fn is_no_attacker(&self) -> bool;

    fn push_event(&mut self, event: EventType);
}

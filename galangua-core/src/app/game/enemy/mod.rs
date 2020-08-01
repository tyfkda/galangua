mod accessor;
mod appearance_manager;
mod attack_manager;
mod ene_shot;
mod enemy;
mod enemy_manager;
mod formation;
mod tractor_beam;
mod traj;
mod traj_command;
mod traj_command_table;

pub use self::accessor::Accessor;
pub use self::enemy::{CaptureState, Enemy};
pub use self::enemy_manager::EnemyManager;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FormationIndex(pub u8, pub u8);  // x, y

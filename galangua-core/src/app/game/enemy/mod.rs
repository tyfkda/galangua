mod accessor;
mod appearance_manager;
mod appearance_table;
mod attack_manager;
mod ene_shot;
mod enemy;
mod enemy_base;
mod enemy_manager;
mod formation;
mod owl;
mod tractor_beam;
mod traj;
pub mod traj_command;
mod traj_command_table;
mod zako;

pub use self::accessor::Accessor;
pub use self::enemy::{Enemy};
pub use self::enemy_manager::EnemyManager;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FormationIndex(pub u8, pub u8);  // x, y

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
    CapturedFighter,
}

#[derive(Debug)]
pub struct DamageResult {
    pub killed: bool,
    pub point: u32,
}

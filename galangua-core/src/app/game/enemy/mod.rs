mod accessor;
pub mod ene_shot;
pub mod enemy;
mod enemy_base;
mod owl;
mod tractor_beam;
pub mod traj;
pub mod traj_command;
pub mod traj_command_table;
mod zako;

pub use self::accessor::Accessor;
pub use self::enemy::Enemy;

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

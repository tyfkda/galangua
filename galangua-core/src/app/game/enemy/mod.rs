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

#[derive(Clone, Copy, PartialEq)]
pub struct FormationIndex(pub u8, pub u8);  // x, y

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
    CapturedFighter,
}

pub struct DamageResult {
    pub point: u32, // > 0 : destroyed.
    pub keep_alive_as_ghost: bool,
}

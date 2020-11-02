mod accessor;
pub mod ene_shot;
pub mod enemy;
mod enemy_base;
mod owl;
mod tractor_beam;
mod zako;

pub use self::accessor::Accessor;
pub use self::enemy::Enemy;

pub struct DamageResult {
    pub point: u32, // > 0 : destroyed.
    pub keep_alive_as_ghost: bool,
}

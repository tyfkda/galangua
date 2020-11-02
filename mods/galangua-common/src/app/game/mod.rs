pub mod appearance_manager;
pub mod appearance_table;
pub mod attack_manager;
pub mod effect_table;
pub mod formation;
pub mod formation_table;
pub mod stage_indicator;
pub mod star_manager;
pub mod tractor_beam_table;
pub mod traj;
pub mod traj_command;
pub mod traj_command_table;

#[derive(Clone, Copy, PartialEq)]
pub struct FormationIndex(pub u8, pub u8);  // x, y

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
    CapturedFighter,
}

#[derive(Clone, Copy)]
pub enum EarnedPointType {
    Point1600,
    Point1000,
    Point800,
    Point400,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CaptureState {
    NoCapture,
    CaptureAttacking,
    Capturing,
    Captured,
    Recapturing,
    Dual,
}

use legion::*;

use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::game::traj::Traj;
use galangua_common::framework::types::Vec2I;

//
#[derive(Clone)]
pub struct Posture(pub Vec2I, pub i32);

//
pub struct Speed(pub i32, pub i32);

//
pub struct CollRect {
    pub offset: Vec2I,
    pub size: Vec2I,
}

//
#[derive(PartialEq)]
pub enum PlayerState {
    Normal,
    Dead,
    Capturing,
    Captured,
    //EscapeCapturing,
    //MoveHomePos,
}
pub struct Player {
    pub state: PlayerState,
    pub count: u32,
}

//
pub struct MyShot;

//
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub formation_index: FormationIndex,
    pub is_formation: bool,
}

//
#[derive(PartialEq)]
pub enum ZakoState {
    Appearance,
    MoveToFormation,
    Formation,
    Attack,
    Troop,
}
pub struct Zako {
    pub state: ZakoState,
    pub traj: Option<Traj>,
}

//
#[derive(Clone, Copy, PartialEq)]
pub enum OwlCaptureAttackPhase {
    Capture,
    CaptureBeam,
    NoCaptureGoOut,
    Capturing,
    CaptureDoneWait,
    CaptureDoneBack,
    CaptureDonePushUp,
}
#[derive(Clone, Copy, PartialEq)]
pub enum OwlState {
    Appearance,
    MoveToFormation,
    Formation,
    TrajAttack,
    CaptureAttack(OwlCaptureAttackPhase),
}
pub enum OwlCapturingState {
    None,
    Attacking,
    BeamTracting,
    //Captured,
    Failed,
}
pub struct Owl {
    pub state: OwlState,
    pub traj: Option<Traj>,
    pub capturing_state: OwlCapturingState,
    pub target_pos: Vec2I,
    pub count: u32,
}

//
pub const TRACTOR_BEAM_SPRITE_COUNT: usize = 29;
#[derive(Copy, Clone, PartialEq)]
pub enum TractorBeamState {
    Opening,
    Full,
    Closing,
    Closed,
    Capturing,
}
pub struct TractorBeam {
    pub pos: Vec2I,
    pub state: TractorBeamState,
    pub count: u32,
    pub color_count: u32,
    pub size_count: i32,
    pub beam_sprites: [Option<Entity>; TRACTOR_BEAM_SPRITE_COUNT],
    pub capturing_player: Option<Entity>,
}

//
const MAX_TROOPS: usize = 3;
pub struct Troops {
    pub members: [Option<(Entity, Vec2I)>; MAX_TROOPS],
}

//
pub struct SequentialSpriteAnime(pub &'static [&'static str], pub u32, pub u32);

//
pub struct SpriteDrawable { pub sprite_name: &'static str, pub offset: Vec2I }

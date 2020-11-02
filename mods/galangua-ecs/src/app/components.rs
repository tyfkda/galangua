use legion::*;

use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::{EnemyType, FormationIndex};
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
    EscapeCapturing,
    MoveHomePos,
}
pub struct Player {
    pub state: PlayerState,
    pub count: u32,
    pub shot_enable: bool,
    pub dual: Option<Entity>,
}

//
pub enum RecapturedFighterState {
    Rotate,
    SlideHorz,
    SlideDown,
    Done,
}
pub struct RecapturedFighter {
    pub state: RecapturedFighterState,
    pub count: u32,
    pub player_entity: Entity,
}

//
pub struct MyShot {
    pub player_entity: Entity,
    pub dual: Option<Entity>,
}

//
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub formation_index: FormationIndex,
    pub is_formation: bool,
}

pub struct EnemyBase {
    pub traj: Option<Traj>,
    pub shot_wait: Option<u32>,
    pub count: u32,
    pub attack_frame_count: u32,
    pub target_pos: Vec2I,
}

//
#[derive(Clone, Copy, PartialEq)]
pub enum ZakoAttackType {
    BeeAttack,
    Traj,
}
#[derive(PartialEq)]
pub enum ZakoState {
    Appearance,
    MoveToFormation,
    Assault(u32),
    Formation,
    Attack(ZakoAttackType),
    Troop,
}
pub struct Zako {
    pub base: EnemyBase,
    pub state: ZakoState,
    //pub traj: Option<Traj>,
    //pub target_pos: Vec2I,
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
    Assault(u32),
    Formation,
    TrajAttack,
    CaptureAttack(OwlCaptureAttackPhase),
}
#[derive(PartialEq)]
pub enum OwlCapturingState {
    None,
    Attacking,
    BeamTracting,
    //Captured,
    Failed,
}
pub struct Owl {
    pub base: EnemyBase,
    pub state: OwlState,
    pub capturing_state: OwlCapturingState,
    pub life: u32,
    //pub traj: Option<Traj>,
    //pub target_pos: Vec2I,
    //pub count: u32,
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
    pub members: [Option<Troop>; MAX_TROOPS],
    pub copy_angle_to_troops: bool,
}
pub struct Troop {
    pub entity: Entity,
    pub offset: Vec2I,
    pub is_guard: bool,
}

//
pub struct EneShot(pub Vec2I);

//
pub struct SequentialSpriteAnime {
    pub sprites: &'static [&'static str],
    pub frame_wait: u32,
    pub delay: u32,
    pub offset: Vec2I,
    pub count: u32,
}

//
pub struct SpriteDrawable {
    pub sprite_name: &'static str,
    pub offset: Vec2I,
}

//
pub struct SpriteColor(pub u8, pub u8, pub u8);

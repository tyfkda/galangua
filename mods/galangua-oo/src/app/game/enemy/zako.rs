use ambassador::Delegate;

use super::enemy::Enemy;
use super::enemy_base::{EnemyBase, EnemyInfo, CoordinateTrait, FormationTrait};
use super::{Accessor, DamageResult};

use crate::app::game::manager::EventType;

use galangua_common::ambassador_impl_Collidable;
use galangua_common::app::consts::*;
use galangua_common::app::game::formation_table::Y_COUNT;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command::TrajCommand;
use galangua_common::app::game::traj_command_table::*;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::util::collision::{CollBox, Collidable};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::framework::RendererTrait;

const BEE_SPRITE_NAMES: [&str; 2] = ["gopher1", "gopher2"];
const BUTTERFLY_SPRITE_NAMES: [&str; 2] = ["dman1", "dman2"];

struct Vtable {
    rush_traj_table: &'static [TrajCommand],
}

const VTABLE: [Vtable; 4] = [
    // Bee
    Vtable {
        rush_traj_table: &BEE_RUSH_ATTACK_TABLE,
    },
    // Butterfly
    Vtable {
        rush_traj_table: &BUTTERFLY_RUSH_ATTACK_TABLE,
    },
    // Owl
    Vtable {
        rush_traj_table: &OWL_RUSH_ATTACK_TABLE,
    },
    // CapturedFighter
    Vtable {
        rush_traj_table: &OWL_RUSH_ATTACK_TABLE,
    },
];

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ZakoAttackType {
    BeeAttack,
    Traj,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ZakoState {
    None,
    Appearance,
    MoveToFormation,
    Assault(u32),
    Formation,
    Attack(ZakoAttackType),
    Troop,
}

#[derive(Delegate)]
#[delegate(Collidable, target="info")]
#[delegate(CoordinateTrait, target="info")]
#[delegate(FormationTrait, target="info")]
pub(super) struct Zako {
    pub(super) enemy_type: EnemyType,
    pub(super) info: EnemyInfo,
    pub(super) base: EnemyBase,
    pub(super) state: ZakoState,
}

impl Zako {
    pub fn new(
        enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
        fi: &FormationIndex,
    ) -> Self {
        Self {
            enemy_type,
            info: EnemyInfo::new(*pos, angle, speed, fi),
            base: EnemyBase::new(),
            state: ZakoState::None,
        }
    }

    pub(super) fn set_state(&mut self, state: ZakoState) {
        self.state = state;
    }

    // update

    fn dispatch_update(&mut self, accessor: &mut dyn Accessor) {
        match self.state {
            ZakoState::None | ZakoState::Troop => {}
            ZakoState::Appearance => {
                if !self.base.update_trajectory(&mut self.info, accessor) {
                    if self.info.formation_index.1 >= Y_COUNT as u8 {  // Assault
                        self.base.set_assault(&mut self.info, accessor);
                        self.set_state(ZakoState::Assault(0));
                    } else {
                        self.set_state(ZakoState::MoveToFormation);
                    }
                }
            }
            ZakoState::MoveToFormation => {
                if !self.base.move_to_formation(&mut self.info, accessor) {
                    self.set_to_formation();
                }
            }
            ZakoState::Assault(phase) => {
                let phase = self.base.update_assault(&mut self.info, phase);
                self.set_state(ZakoState::Assault(phase));
            }
            ZakoState::Formation => { self.info.update_formation(accessor); }
            ZakoState::Attack(t) => {
                self.base.update_attack(&self.info, true, accessor);
                match t {
                    ZakoAttackType::BeeAttack => self.update_bee_attack(accessor),
                    ZakoAttackType::Traj => self.update_attack_traj(accessor),
                }
            }
        }
    }

    fn update_bee_attack(&mut self, accessor: &mut dyn Accessor) {
        if !self.base.update_trajectory(&mut self.info, accessor) {
            if accessor.is_rush() {
                let flip_x = self.info.formation_index.0 >= 5;
                let mut traj = Traj::new(&BEE_ATTACK_RUSH_CONT_TABLE, &ZERO_VEC, flip_x,
                                         self.info.formation_index);
                traj.set_pos(&self.info.pos);

                self.base.traj = Some(traj);
                self.set_state(ZakoState::Attack(ZakoAttackType::Traj));
            } else {
                self.set_state(ZakoState::MoveToFormation);
            }
        }
    }

    fn update_attack_traj(&mut self, accessor: &mut dyn Accessor) {
        if !self.base.update_trajectory(&mut self.info, accessor) {
            if self.enemy_type == EnemyType::CapturedFighter {
                self.base.disappeared = true;
            } else if accessor.is_rush() {
                // Rush mode: Continue attacking
                let table = VTABLE[self.enemy_type as usize].rush_traj_table;
                self.base.rush_attack(&self.info, table);
                accessor.play_se(CH_ATTACK, SE_ATTACK_START);
            } else {
                self.set_state(ZakoState::MoveToFormation);
            }
        }
    }

    // start_attack

    fn start_bee_attack(&mut self) {
        let flip_x = self.info.formation_index.0 >= 5;
        let mut traj = Traj::new(&BEE_ATTACK_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
        traj.set_pos(&self.info.pos);

        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.base.traj = Some(traj);
        self.set_state(ZakoState::Attack(ZakoAttackType::BeeAttack));
    }

    fn start_butterfly_attack(&mut self) {
        let flip_x = self.info.formation_index.0 >= 5;
        let mut traj = Traj::new(&BUTTERFLY_ATTACK_TABLE, &ZERO_VEC, flip_x,
                                 self.info.formation_index);
        traj.set_pos(&self.info.pos);

        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.base.traj = Some(traj);
        self.set_state(ZakoState::Attack(ZakoAttackType::Traj));
    }

    fn start_captured_fighter_attack(&mut self) {
        let flip_x = self.info.formation_index.0 >= 5;
        let mut traj = Traj::new(&OWL_ATTACK_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
        traj.set_pos(&self.info.pos);

        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.base.traj = Some(traj);
        self.set_state(ZakoState::Attack(ZakoAttackType::Traj));
    }

    // set_damage

    fn bee_set_damage(&mut self) -> DamageResult {
        let point = self.calc_point();
        DamageResult { point, keep_alive_as_ghost: false }
    }

    fn captured_fighter_set_damage(&mut self, accessor: &mut dyn Accessor) -> DamageResult {
        accessor.push_event(EventType::CapturedFighterDestroyed);
        let point = self.calc_point();
        DamageResult { point, keep_alive_as_ghost: false }
    }

    fn calc_point(&self) -> u32 {
        match self.enemy_type {
            EnemyType::Bee => {
                if self.is_formation() { 50 } else { 100 }
            }
            EnemyType::Butterfly => {
                if self.is_formation() { 80 } else { 160 }
            }
            EnemyType::CapturedFighter => {
                if self.is_formation() { 500 } else { 1000 }
            }
            _ => { panic!("Illegal"); }
        }
    }
}

impl Enemy for Zako {
    fn update(&mut self, accessor: &mut dyn Accessor) -> bool {
        self.dispatch_update(accessor);
        self.info.forward();
        !self.base.disappeared
    }

    fn draw(&self, renderer: &mut dyn RendererTrait, pat: usize) {
        let sprite = match self.enemy_type {
            EnemyType::Bee => BEE_SPRITE_NAMES[pat],
            EnemyType::Butterfly => BUTTERFLY_SPRITE_NAMES[pat],
            EnemyType::CapturedFighter => "rustacean_captured",
            _ => { panic!("Illegal"); }
        };
        self.draw_sprite(renderer, sprite, &Vec2I::new(8, 8));
    }

    fn is_formation(&self) -> bool { self.state == ZakoState::Formation }

    fn set_damage(&mut self, _power: u32, accessor: &mut dyn Accessor) -> DamageResult {
        self.info.explode(accessor, self.enemy_type);
        if self.enemy_type == EnemyType::CapturedFighter {
            accessor.play_se(CH_JINGLE, SE_BOMB_CAPTURED);
        } else {
            accessor.play_se(CH_BOMB, SE_BOMB_ZAKO);
        }
        match self.enemy_type {
            EnemyType::Bee | EnemyType::Butterfly => self.bee_set_damage(),
            EnemyType::CapturedFighter => self.captured_fighter_set_damage(accessor),
            _ => { panic!("Illegal"); }
        }
    }

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>) {
        self.info.pos += add;
        if let Some(angle) = angle_opt {
            self.info.angle = angle;
        }
    }

    fn start_attack(&mut self, _capture_attack: bool, accessor: &mut dyn Accessor) {
        match self.enemy_type {
            EnemyType::Bee => self.start_bee_attack(),
            EnemyType::Butterfly => self.start_butterfly_attack(),
            EnemyType::CapturedFighter => self.start_captured_fighter_attack(),
            _ => { panic!("Illgal"); }
        }

        accessor.play_se(CH_ATTACK, SE_ATTACK_START);
    }

    fn set_to_troop(&mut self) {
        self.set_state(ZakoState::Troop);
    }
    fn set_to_formation(&mut self) {
        self.base.set_to_formation(&mut self.info);
        self.set_state(ZakoState::Formation);
    }

    #[cfg(debug_assertions)]
    fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool) {
        self.base.set_table_attack(&mut self.info, traj_command_vec, flip_x);
        self.set_state(ZakoState::Attack(ZakoAttackType::Traj));
    }
}

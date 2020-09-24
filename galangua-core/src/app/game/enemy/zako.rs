use super::enemy::Enemy;
use super::enemy_base::{EnemyBase, EnemyInfo};
use super::traj::Traj;
use super::traj_command::TrajCommand;
use super::traj_command_table::*;
use super::{Accessor, DamageResult, EnemyType, FormationIndex};

use crate::app::consts::*;
use crate::app::game::manager::formation::Y_COUNT;
use crate::app::game::manager::EventType;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::framework::RendererTrait;
use crate::util::math::{quantize_angle, round_vec};

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
pub(super) enum ZakoAttackPhase {
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
    Attack(ZakoAttackPhase),
    Troop,
}

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

    //// update

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
            ZakoState::Attack(phase) => {
                match phase {
                    ZakoAttackPhase::BeeAttack => { self.update_bee_attack(accessor) }
                    ZakoAttackPhase::Traj => { self.update_attack_traj(accessor); }
                }
            }
        }
    }

    fn update_bee_attack(&mut self, accessor: &mut dyn Accessor) {
        self.base.update_attack(&self.info, accessor);
        if !self.base.update_trajectory(&mut self.info, accessor) {
            if accessor.is_rush() {
                let flip_x = self.info.formation_index.0 >= 5;
                let mut traj = Traj::new(&BEE_ATTACK_RUSH_CONT_TABLE, &ZERO_VEC, flip_x,
                                         self.info.formation_index);
                traj.set_pos(&self.info.pos);

                self.base.traj = Some(traj);
                self.set_state(ZakoState::Attack(ZakoAttackPhase::Traj));
            } else {
                self.set_state(ZakoState::MoveToFormation);
            }
        }
    }

    fn update_attack_traj(&mut self, accessor: &mut dyn Accessor) {
        self.base.update_attack(&self.info, accessor);
        if !self.base.update_trajectory(&mut self.info, accessor) {
            if self.enemy_type == EnemyType::CapturedFighter {
                self.base.disappeared = true;
            } else if accessor.is_rush() {
                // Rush mode: Continue attacking
                let table = VTABLE[self.enemy_type as usize].rush_traj_table;
                self.base.rush_attack(&mut self.info, table);
                accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
            } else {
                self.set_state(ZakoState::MoveToFormation);
            }
        }
    }

    //// set_attack

    fn set_bee_attack(&mut self) {
        let flip_x = self.info.formation_index.0 >= 5;
        let mut traj = Traj::new(&BEE_ATTACK_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
        traj.set_pos(&self.info.pos);

        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.base.traj = Some(traj);
        self.set_state(ZakoState::Attack(ZakoAttackPhase::BeeAttack));
    }

    fn set_butterfly_attack(&mut self) {
        let flip_x = self.info.formation_index.0 >= 5;
        let mut traj = Traj::new(&BUTTERFLY_ATTACK_TABLE, &ZERO_VEC, flip_x,
                                 self.info.formation_index);
        traj.set_pos(&self.info.pos);

        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.base.traj = Some(traj);
        self.set_state(ZakoState::Attack(ZakoAttackPhase::Traj));
    }

    fn set_captured_fighter_attack(&mut self) {
        let flip_x = self.info.formation_index.0 >= 5;
        let mut traj = Traj::new(&OWL_ATTACK_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
        traj.set_pos(&self.info.pos);

        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.base.traj = Some(traj);
        self.set_state(ZakoState::Attack(ZakoAttackPhase::Traj));
    }

    //// set_damage

    fn bee_set_damage(&mut self) -> DamageResult {
        let point = self.calc_point();
        DamageResult { killed: true, point }
    }

    fn captured_fighter_set_damage(&mut self, accessor: &mut dyn Accessor) -> DamageResult {
        accessor.push_event(EventType::CapturedFighterDestroyed);
        let point = self.calc_point();
        DamageResult { killed: true, point }
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

impl Collidable for Zako {
    fn get_collbox(&self) -> Option<CollBox> { Some(self.info.get_collbox()) }
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

        let angle = quantize_angle(self.info.angle, ANGLE_DIV);
        let pos = round_vec(&self.info.pos);
        renderer.draw_sprite_rot(sprite, &(&pos + &Vec2I::new(-8, -8)), angle, None);
    }

    fn pos(&self) -> &Vec2I { &self.info.pos }
    fn set_pos(&mut self, pos: &Vec2I) { self.info.pos = *pos; }

    fn is_formation(&self) -> bool { self.state == ZakoState::Formation }

    fn can_capture_attack(&self) -> bool { false }
    fn is_captured_fighter(&self) -> bool { self.enemy_type == EnemyType::CapturedFighter }
    fn formation_index(&self) -> &FormationIndex { &self.info.formation_index }

    fn set_damage(&mut self, _power: u32, accessor: &mut dyn Accessor) -> DamageResult {
        accessor.push_event(EventType::EnemyExplosion(
            self.info.pos, self.info.angle, self.enemy_type));
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

    fn set_attack(&mut self, _capture_attack: bool, accessor: &mut dyn Accessor) {
        match self.enemy_type {
            EnemyType::Bee => self.set_bee_attack(),
            EnemyType::Butterfly => self.set_butterfly_attack(),
            EnemyType::CapturedFighter => self.set_captured_fighter_attack(),
            _ => { panic!("Illgal"); }
        }

        accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
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
        self.set_state(ZakoState::Attack(ZakoAttackPhase::Traj));
    }
}

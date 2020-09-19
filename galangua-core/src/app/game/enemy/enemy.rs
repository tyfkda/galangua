use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::formation::Y_COUNT;
use super::tractor_beam::TractorBeam;
use super::traj::Traj;
use super::traj_command::TrajCommand;
use super::traj_command_table::*;
use super::{Accessor, FormationIndex};

use crate::app::consts::*;
use crate::app::game::{EventQueue, EventType};
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::framework::RendererTrait;
use crate::util::math::{
    atan2_lut, calc_velocity, clamp, diff_angle, normalize_angle, quantize_angle, round_vec, square,
    ANGLE, ONE, ONE_BIT};

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

pub trait Enemy : Collidable {
    fn update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) -> bool;
    fn draw(&self, renderer: &mut dyn RendererTrait, pat: usize);

    fn pos(&self) -> &Vec2I;
    fn set_pos(&mut self, pos: &Vec2I);

    fn is_formation(&self) -> bool;

    fn can_capture_attack(&self) -> bool;
    fn is_captured_fighter(&self) -> bool;
    fn formation_index(&self) -> &FormationIndex;

    fn set_damage(
        &mut self, power: u32, accessor: &mut dyn Accessor, event_queue: &mut EventQueue,
    ) -> DamageResult;

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>);

    fn set_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor, event_queue: &mut EventQueue);
    fn set_to_troop(&mut self);
    fn set_to_formation(&mut self);

    #[cfg(debug_assertions)]
    fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool);
}

pub struct EnemyInfo {
    pos: Vec2I,
    angle: i32,
    speed: i32,
    vangle: i32,
    formation_index: FormationIndex,
}

impl EnemyInfo {
    pub fn new(pos: Vec2I, angle: i32, speed: i32, fi: &FormationIndex) -> Self {
        Self {
            pos,
            angle,
            speed,
            vangle: 0,
            formation_index: *fi,
        }
    }

    fn warp(&mut self, offset: Vec2I) {
        self.pos += offset;
    }

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>) {
        self.pos += *add;
        if let Some(angle) = angle_opt {
            self.angle = angle;
        }
    }

    fn update_formation(&mut self, accessor: &mut dyn Accessor) {
        self.pos = accessor.get_formation_pos(&self.formation_index);

        let ang = ANGLE * ONE / 128;
        self.angle -= clamp(self.angle, -ang, ang);
    }

    //impl Collidable for EnemyBase
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: &round_vec(&self.pos) - &Vec2I::new(6, 6),
            size: Vec2I::new(12, 12),
        }
    }
}

pub struct EnemyBase {
    traj: Option<Traj>,
    shot_wait: Option<u32>,
    count: u32,
    attack_frame_count: u32,
    target_pos: Vec2I,
    disappeared: bool,
}

impl EnemyBase {
    pub fn new() -> Self {
        Self {
            traj: None,
            shot_wait: None,
            count: 0,
            attack_frame_count: 0,
            target_pos: ZERO_VEC,
            disappeared: false,
        }
    }

    pub fn update_attack(&mut self, info: &EnemyInfo, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) -> bool {
        self.attack_frame_count += 1;

        let stage_no = accessor.get_stage_no();
        let shot_count = std::cmp::min(2 + stage_no / 8 , 5) as u32;
        let shot_interval = 20 - shot_count * 2;

        if self.attack_frame_count <= shot_interval * shot_count && self.attack_frame_count % shot_interval == 0 {
            event_queue.push(EventType::EneShot(info.pos));
            true
        } else {
            false
        }
    }

    fn move_to_formation(&mut self, info: &mut EnemyInfo, accessor: &dyn Accessor) -> bool {
        let target = accessor.get_formation_pos(&info.formation_index);
        let diff = &target - &info.pos;
        let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
        if sq_distance > square(info.speed >> (ONE_BIT / 2)) {
            let dlimit: i32 = info.speed * 5 / 3;
            let target_angle = atan2_lut(-diff.y, diff.x);
            let d = diff_angle(target_angle, info.angle);
            info.angle += clamp(d, -dlimit, dlimit);
            info.vangle = 0;
            true
        } else {
            info.pos = target;
            info.speed = 0;
            false
        }
    }

    fn rush_attack(&mut self, info: &EnemyInfo, table: &'static [TrajCommand]) {
        let flip_x = info.formation_index.0 >= 5;
        let mut traj = Traj::new(table, &ZERO_VEC, flip_x, info.formation_index);
        traj.set_pos(&info.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
    }

    fn set_assault(&mut self, info: &mut EnemyInfo, accessor: &dyn Accessor) {
        let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
        let target_pos = [
            Some(*accessor.get_player_pos()),
            accessor.get_dual_player_pos(),
        ];
        let count = target_pos.iter().flat_map(|x| x).count();
        let target: &Vec2I = target_pos.iter()
            .flat_map(|x| x).nth(rng.gen_range(0, count)).unwrap();

        self.target_pos = *target;
        info.vangle = 0;
    }

    //// Update

    fn update_trajectory(&mut self, info: &mut EnemyInfo, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) -> bool {
        if let Some(traj) = &mut self.traj {
            let cont = traj.update(accessor);

            info.pos = traj.pos();
            info.angle = traj.angle();
            info.speed = traj.speed;
            info.vangle = traj.vangle;
            if let Some(wait) = traj.is_shot() {
                self.shot_wait = Some(wait);
            }

            if let Some(wait) = self.shot_wait {
                if wait > 0 {
                    self.shot_wait = Some(wait - 1);
                } else {
                    event_queue.push(EventType::EneShot(info.pos));
                    self.shot_wait = None;
                }
            }

            if cont {
                return true;
            }
        }

        self.traj = None;
        false
    }

    fn update_assault(&mut self, info: &mut EnemyInfo, mut phase: u32) -> u32 {
        match phase {
            0 => {
                let target = &self.target_pos;
                let diff = target - &info.pos;

                const DLIMIT: i32 = 5 * ONE;
                let target_angle = atan2_lut(-diff.y, diff.x);
                let d = diff_angle(target_angle, info.angle);
                if d < -DLIMIT {
                    info.angle -= DLIMIT;
                } else if d > DLIMIT {
                    info.angle += DLIMIT;
                } else {
                    info.angle += d;
                    phase = 1;
                }
            }
            1 | _ => {
                if info.pos.y >= (HEIGHT + 8) * ONE {
                    self.disappeared = true;
                    phase = 2;
                }
            }
        }
        phase
    }

    //impl Enemy for EnemyBase
    fn set_to_formation(&mut self, info: &mut EnemyInfo) {
        info.speed = 0;
        info.angle = normalize_angle(info.angle);
        info.vangle = 0;
    }

    #[cfg(debug_assertions)]
    fn set_table_attack(&mut self, info: &mut EnemyInfo, traj_command_vec: Vec<TrajCommand>, flip_x: bool) {
        let mut traj = Traj::new_with_vec(traj_command_vec, &ZERO_VEC, flip_x, info.formation_index);
        traj.set_pos(&info.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
    }
}

////////////////////////////////////////////////

struct EnemyVtable {
    rush_traj_table: &'static [TrajCommand],
}

const BEE_SPRITE_NAMES: [&str; 2] = ["gopher1", "gopher2"];
const BUTTERFLY_SPRITE_NAMES: [&str; 2] = ["dman1", "dman2"];
const OWL_SPRITE_NAMES: [&str; 4] = ["cpp11", "cpp12", "cpp21", "cpp22"];

const ENEMY_VTABLE: [EnemyVtable; 4] = [
    // Bee
    EnemyVtable {
        rush_traj_table: &BEE_RUSH_ATTACK_TABLE,
    },
    // Butterfly
    EnemyVtable {
        rush_traj_table: &BUTTERFLY_RUSH_ATTACK_TABLE,
    },
    // Owl
    EnemyVtable {
        rush_traj_table: &OWL_RUSH_ATTACK_TABLE,
    },
    // CapturedFighter
    EnemyVtable {
        rush_traj_table: &OWL_RUSH_ATTACK_TABLE,
    },
];

////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZakoAttackPhase {
    BeeAttack,
    Traj,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZakoState {
    None,
    Appearance,
    MoveToFormation,
    Assault(u32),
    Formation,
    Attack(ZakoAttackPhase),
    Troop,
}

pub struct Zako {
    vtable: &'static EnemyVtable,
    enemy_type: EnemyType,
    info: EnemyInfo,
    base: EnemyBase,
    state: ZakoState,
}

impl Zako {
    pub fn new(
        enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
        fi: &FormationIndex,
    ) -> Self {
        let vtable = &ENEMY_VTABLE[enemy_type as usize];

        Self {
            vtable,
            enemy_type,
            info: EnemyInfo::new(*pos, angle, speed, fi),
            base: EnemyBase::new(),
            state: ZakoState::None,
        }
    }

    fn set_state(&mut self, state: ZakoState) {
        self.state = state;
    }

    //// update

    fn dispatch_update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        match self.state {
            ZakoState::None | ZakoState::Troop => {}
            ZakoState::Appearance => {
                if !self.base.update_trajectory(&mut self.info, accessor, event_queue) {
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
                    ZakoAttackPhase::BeeAttack => { self.update_bee_attack(accessor, event_queue) }
                    ZakoAttackPhase::Traj => { self.update_attack_traj(accessor, event_queue); }
                }
            }
        }
    }

    fn update_bee_attack(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        self.base.update_attack(&self.info, accessor, event_queue);
        if !self.base.update_trajectory(&mut self.info, accessor, event_queue) {
            if accessor.is_rush() {
                let flip_x = self.info.formation_index.0 >= 5;
                let mut traj = Traj::new(&BEE_ATTACK_RUSH_CONT_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
                traj.set_pos(&self.info.pos);

                self.base.traj = Some(traj);
                self.set_state(ZakoState::Attack(ZakoAttackPhase::Traj));

                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
            } else {
                self.set_state(ZakoState::MoveToFormation);
            }
        }
    }

    fn update_attack_traj(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        self.base.update_attack(&self.info, accessor, event_queue);
        if !self.base.update_trajectory(&mut self.info, accessor, event_queue) {
            if self.enemy_type == EnemyType::CapturedFighter {
                self.base.disappeared = true;
            } else if accessor.is_rush() {
                // Rush mode: Continue attacking
                let table = self.vtable.rush_traj_table;
                self.base.rush_attack(&mut self.info, table);
                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
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
        let mut traj = Traj::new(&BUTTERFLY_ATTACK_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
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

    fn captured_fighter_set_damage(&mut self, event_queue: &mut EventQueue) -> DamageResult {
        event_queue.push(EventType::CapturedFighterDestroyed);
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
    fn update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) -> bool {
        self.dispatch_update(accessor, event_queue);

        self.info.pos += calc_velocity(self.info.angle + self.info.vangle / 2, self.info.speed);
        self.info.angle += self.info.vangle;

        !self.base.disappeared
    }

    fn draw(&self, renderer: &mut dyn RendererTrait, pat: usize) {
        let sprite = match self.enemy_type {
            EnemyType::Bee => { BEE_SPRITE_NAMES[pat] }
            EnemyType::Butterfly => { BUTTERFLY_SPRITE_NAMES[pat] }
            EnemyType::CapturedFighter => { "rustacean_captured" }
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

    fn set_damage(
        &mut self, _power: u32, _accessor: &mut dyn Accessor, event_queue: &mut EventQueue,
    ) -> DamageResult {
        event_queue.push(EventType::EnemyExplosion(self.info.pos, self.info.angle, self.enemy_type));
        match self.enemy_type {
            EnemyType::Bee | EnemyType::Butterfly => { self.bee_set_damage() }
            EnemyType::CapturedFighter => { self.captured_fighter_set_damage(event_queue) }
            _ => { panic!("Illegal"); }
        }
    }

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>) {
        self.info.update_troop(add, angle_opt);
    }

    fn set_attack(&mut self, _capture_attack: bool, _accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        match self.enemy_type {
            EnemyType::Bee => { self.set_bee_attack(); }
            EnemyType::Butterfly => { self.set_butterfly_attack(); }
            EnemyType::CapturedFighter => { self.set_captured_fighter_attack(); }
            _ => { panic!("Illgal"); }
        }

        event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
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

////////////////////////////////////////////////

const MAX_TROOPS: usize = 3;
const OWL_DESTROY_SHOT_WAIT: u32 = 3 * 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OwlAttackPhase {
    Traj,
    Capture,
    CaptureBeam,
    NoCaptureGoOut,
    CaptureStart,
    CaptureCloseBeam,
    CaptureDoneWait,
    CaptureDoneBack,
    CaptureDonePushUp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OwlState {
    None,
    Appearance,
    MoveToFormation,
    Assault(u32),
    Formation,
    Attack(OwlAttackPhase),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CapturingState {
    None,
    Attacking,
    BeamTracting,
}

pub struct Owl {
    info: EnemyInfo,
    base: EnemyBase,
    state: OwlState,
    life: u32,
    tractor_beam: Option<TractorBeam>,
    capturing_state: CapturingState,
    troops: [Option<FormationIndex>; MAX_TROOPS],
    copy_angle_to_troops: bool,
}

impl Owl {
    pub fn new(
        pos: &Vec2I, angle: i32, speed: i32,
        fi: &FormationIndex,
    ) -> Self {
        Owl {
            info: EnemyInfo::new(*pos, angle, speed, fi),
            base: EnemyBase::new(),
            state: OwlState::None,
            life: 2,
            tractor_beam: None,
            capturing_state: CapturingState::None,
            troops: Default::default(),
            copy_angle_to_troops: true,
        }
    }

    fn set_state(&mut self, state: OwlState) {
        self.state = state;
    }

    fn calc_point(&self) -> u32 {
        if self.is_formation() {
            150
        } else {
            let cap_fi = FormationIndex(self.info.formation_index.0, self.info.formation_index.1 - 1);
            let count = self.troops.iter().flat_map(|x| x)
                .filter(|index| **index != cap_fi)
                .count();
            (1 << count) * 400
        }
    }

    fn live_troops(&self, accessor: &dyn Accessor) -> bool {
        self.troops.iter().flat_map(|x| x)
            .filter_map(|index| accessor.get_enemy_at(index))
            .any(|enemy| !enemy.is_captured_fighter())
    }

    fn add_troop(&mut self, formation_index: FormationIndex) -> bool {
        if let Some(slot) = self.troops.iter_mut().find(|x| x.is_none()) {
            *slot = Some(formation_index);
            true
        } else {
            false
        }
    }

    fn choose_troops(&mut self, accessor: &mut dyn Accessor) {
        let base = &self.info.formation_index;
        let indices = [
            FormationIndex(base.0 - 1, base.1 + 1),
            FormationIndex(base.0 + 1, base.1 + 1),
            FormationIndex(base.0, base.1 - 1),
        ];
        for index in indices.iter() {
            if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                if enemy.is_formation() {
                    self.add_troop(*index);
                }
            }
        }
        self.troops.iter().flat_map(|x| x).for_each(|index| {
            if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                enemy.set_to_troop();
            }
        });
    }

    fn update_troops(&mut self, add: &Vec2I, angle_opt: Option<i32>, accessor: &mut dyn Accessor) {
        for troop_opt in self.troops.iter_mut() {
            if let Some(formation_index) = troop_opt {
                if let Some(troop) = accessor.get_enemy_at_mut(formation_index) {
                    troop.update_troop(add, angle_opt);
                } else {
                    //*troop_opt = None;
                }
            }
        }
    }

    fn release_troops(&mut self, accessor: &mut dyn Accessor) {
        for troop_opt in self.troops.iter_mut().filter(|x| x.is_some()) {
            let index = &troop_opt.unwrap();
            if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                enemy.set_to_formation();
            }
            *troop_opt = None;
        }
    }

    fn remove_destroyed_troops(&mut self, accessor: &mut dyn Accessor) {
        for troop_opt in self.troops.iter_mut().filter(|x| x.is_some()) {
            let index = &troop_opt.unwrap();
            if accessor.get_enemy_at(index).is_none() {
                *troop_opt = None;
            }
        }
    }

    fn dispatch_update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        match self.state {
            OwlState::None => {}
            OwlState::Appearance => {
                if !self.base.update_trajectory(&mut self.info, accessor, event_queue) {
                    if self.info.formation_index.1 >= Y_COUNT as u8 {  // Assault
                        self.base.set_assault(&mut self.info, accessor);
                        self.set_state(OwlState::Assault(0));
                    } else {
                        self.set_state(OwlState::MoveToFormation);
                    }
                }
            }
            OwlState::MoveToFormation => {
                if !self.base.move_to_formation(&mut self.info, accessor) {
                    self.capturing_state = CapturingState::None;
                    self.release_troops(accessor);
                    self.set_to_formation();
                }
            }
            OwlState::Assault(phase) => {
                let phase = self.base.update_assault(&mut self.info, phase);
                self.set_state(OwlState::Assault(phase));
            }
            OwlState::Formation => { self.info.update_formation(accessor); }
            OwlState::Attack(phase) => {
                match phase {
                    OwlAttackPhase::Traj => { self.update_attack_traj(accessor, event_queue); }
                    OwlAttackPhase::Capture => { self.update_attack_capture(); }
                    OwlAttackPhase::CaptureBeam => { self.update_attack_capture_beam(accessor, event_queue); }
                    OwlAttackPhase::NoCaptureGoOut => { self.update_attack_capture_go_out(accessor, event_queue); }
                    OwlAttackPhase::CaptureStart => { self.update_attack_capture_start(accessor); }
                    OwlAttackPhase::CaptureCloseBeam => { self.update_attack_capture_close_beam(event_queue); }
                    OwlAttackPhase::CaptureDoneWait => { self.update_attack_capture_done_wait(); }
                    OwlAttackPhase::CaptureDoneBack => { self.update_attack_capture_back(accessor); }
                    OwlAttackPhase::CaptureDonePushUp => { self.update_attack_capture_push_up(accessor, event_queue); }
                }
            }
        }
    }

    fn update_attack_capture(&mut self) {
        const DLIMIT: i32 = 4 * ONE;
        let dpos = &self.base.target_pos - &self.info.pos;
        let target_angle = atan2_lut(-dpos.y, dpos.x);
        let ang_limit = ANGLE * ONE / 2 - ANGLE * ONE * 30 / 360;
        let target_angle = if target_angle >= 0 {
            std::cmp::max(target_angle, ang_limit)
        } else {
            std::cmp::min(target_angle, -ang_limit)
        };
        let mut d = diff_angle(target_angle, self.info.angle);
        if self.info.vangle > 0 && d < 0 {
            d += ANGLE * ONE;
        } else if self.info.vangle < 0 && d > 0 {
            d -= ANGLE * ONE;
        }
        if d >= -DLIMIT && d < DLIMIT {
            self.info.angle = target_angle;
            self.info.vangle = 0;
        }

        if self.info.pos.y >= self.base.target_pos.y {
            self.info.pos.y = self.base.target_pos.y;
            self.info.speed = 0;
            self.info.angle = ANGLE / 2 * ONE;
            self.info.vangle = 0;

            self.tractor_beam = Some(TractorBeam::new(&(&self.info.pos + &Vec2I::new(0, 8 * ONE))));

            self.set_state(OwlState::Attack(OwlAttackPhase::CaptureBeam));
            self.base.count = 0;
        }
    }
    fn update_attack_capture_beam(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        if let Some(tractor_beam) = &mut self.tractor_beam {
            if tractor_beam.closed() {
                self.tractor_beam = None;
                self.info.speed = 5 * ONE / 2;
                self.set_state(OwlState::Attack(OwlAttackPhase::NoCaptureGoOut));
            } else if accessor.can_player_capture() &&
                      tractor_beam.can_capture(accessor.get_player_pos())
            {
                event_queue.push(EventType::CapturePlayer(&self.info.pos + &Vec2I::new(0, 16 * ONE)));
                tractor_beam.start_capture();
                self.capturing_state = CapturingState::BeamTracting;
                self.set_state(OwlState::Attack(OwlAttackPhase::CaptureStart));
                self.base.count = 0;
            }
        }
    }
    fn update_attack_capture_go_out(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        if self.info.pos.y >= (HEIGHT + 8) * ONE {
            let target_pos = accessor.get_formation_pos(&self.info.formation_index);
            let offset = Vec2I::new(target_pos.x - self.info.pos.x, (-32 - (HEIGHT + 8)) * ONE);
            self.info.warp(offset);

            if accessor.is_rush() {
                self.rush_attack();
                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
            } else {
                self.set_state(OwlState::MoveToFormation);
                self.capturing_state = CapturingState::None;
                event_queue.push(EventType::EndCaptureAttack);
            }
        }
    }
    fn update_attack_capture_start(&mut self, accessor: &mut dyn Accessor) {
        if accessor.is_player_capture_completed() {
            self.tractor_beam.as_mut().unwrap().close_capture();
            self.set_state(OwlState::Attack(OwlAttackPhase::CaptureCloseBeam));
            self.base.count = 0;
        }
    }
    fn update_attack_capture_close_beam(&mut self, event_queue: &mut EventQueue) {
        if let Some(tractor_beam) = &self.tractor_beam {
            if tractor_beam.closed() {
                let fi = FormationIndex(self.info.formation_index.0, self.info.formation_index.1 - 1);
                event_queue.push(EventType::SpawnCapturedFighter(
                    &self.info.pos + &Vec2I::new(0, 16 * ONE), fi));

                self.add_troop(fi);

                self.tractor_beam = None;
                self.capturing_state = CapturingState::None;
                event_queue.push(EventType::CapturePlayerCompleted);

                self.copy_angle_to_troops = false;
                self.set_state(OwlState::Attack(OwlAttackPhase::CaptureDoneWait));
                self.base.count = 0;
            }
        }
    }
    fn update_attack_capture_done_wait(&mut self) {
        self.base.count += 1;
        if self.base.count >= 120 {
            self.info.speed = 5 * ONE / 2;
            self.set_state(OwlState::Attack(OwlAttackPhase::CaptureDoneBack));
        }
    }
    fn update_attack_capture_back(&mut self, accessor: &mut dyn Accessor) {
        if !self.base.move_to_formation(&mut self.info, accessor) {
            self.info.speed = 0;
            self.info.angle = normalize_angle(self.info.angle);
            self.set_state(OwlState::Attack(OwlAttackPhase::CaptureDonePushUp));
        }
    }
    fn update_attack_capture_push_up(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        let ang = ANGLE * ONE / 128;
        self.info.angle -= clamp(self.info.angle, -ang, ang);

        let fi = FormationIndex(self.info.formation_index.0, self.info.formation_index.1 - 1);
        let mut done = false;
        if let Some(captured_fighter) = accessor.get_enemy_at_mut(&fi) {
            let mut pos = *captured_fighter.pos();
            pos.y -= 1 * ONE;
            let topy = self.info.pos.y - 16 * ONE;
            if pos.y <= topy {
                pos.y = topy;
                done = true;
            }
            captured_fighter.set_pos(&pos);
        }
        if done {
            event_queue.push(EventType::CaptureSequenceEnded);
            self.release_troops(accessor);
            self.set_to_formation();
        }
    }

    fn update_attack_traj(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        self.update_attack(accessor, event_queue);
        if !self.base.update_trajectory(&mut self.info, accessor, event_queue) {
            if accessor.is_rush() {
                // Rush mode: Continue attacking
                self.remove_destroyed_troops(accessor);
                self.rush_attack();
                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
            } else {
                self.set_state(OwlState::MoveToFormation);
            }
        }
    }

    fn update_attack(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        if self.base.update_attack(&mut self.info, accessor, event_queue) {
            for troop_fi in self.troops.iter().flat_map(|x| x) {
                if let Some(enemy) = accessor.get_enemy_at(troop_fi) {
                    event_queue.push(EventType::EneShot(*enemy.pos()));
                }
            }
        }
    }

    fn owl_set_damage(
        &mut self, power: u32, accessor: &mut dyn Accessor, event_queue: &mut EventQueue,
    ) -> DamageResult {
        if self.life > power {
            self.life -= power;
            DamageResult { killed: false, point: 0 }
        } else {
            let mut killed = true;
            self.life = 0;
            if self.live_troops(accessor) {
                killed = false;  // Keep alive as a ghost.
            }
            let point = self.calc_point();

            // Release capturing.
            match self.capturing_state {
                CapturingState::None => {
                    let fi = FormationIndex(self.info.formation_index.0, self.info.formation_index.1 - 1);
                    if self.troops.iter().flat_map(|x| x)
                        .find(|index| **index == fi).is_some()
                    {
                        event_queue.push(EventType::RecapturePlayer(fi));
                    }
                }
                CapturingState::Attacking => {
                    event_queue.push(EventType::EndCaptureAttack);
                }
                CapturingState::BeamTracting => {
                    event_queue.push(EventType::EscapeCapturing);
                }
            }
            self.capturing_state = CapturingState::None;

            accessor.pause_enemy_shot(OWL_DESTROY_SHOT_WAIT);

            event_queue.push(EventType::EnemyExplosion(self.info.pos, self.info.angle, EnemyType::Owl));

            DamageResult { killed, point }
        }
    }

    fn rush_attack(&mut self) {
        let table = &OWL_RUSH_ATTACK_TABLE;
        self.base.rush_attack(&mut self.info, table);
        self.set_state(OwlState::Attack(OwlAttackPhase::Traj));
    }
}

impl Collidable for Owl {
    fn get_collbox(&self) -> Option<CollBox> {
        if self.life > 0 {
            Some(self.info.get_collbox())
        } else {
            None
        }
    }
}

impl Enemy for Owl {
    fn update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) -> bool {
        let prev_pos = self.info.pos;

        self.dispatch_update(accessor, event_queue);

        self.info.pos += calc_velocity(self.info.angle + self.info.vangle / 2, self.info.speed);
        self.info.angle += self.info.vangle;

        let angle_opt = if self.copy_angle_to_troops { Some(self.info.angle) } else { None };
        self.update_troops(&(&self.info.pos - &prev_pos), angle_opt, accessor);

        if let Some(tractor_beam) = &mut self.tractor_beam {
            tractor_beam.update();
        }

        if self.life == 0 && !self.base.disappeared && !self.live_troops(accessor) {
            self.base.disappeared = true;
        }
        !self.base.disappeared
    }

    fn draw(&self, renderer: &mut dyn RendererTrait, pat: usize) {
        if self.life == 0 {
            return;
        }

        let pat = if self.life <= 1 { pat + 2 } else { pat };
        let sprite = OWL_SPRITE_NAMES[pat as usize];

        let angle = quantize_angle(self.info.angle, ANGLE_DIV);
        let pos = round_vec(&self.info.pos);
        renderer.draw_sprite_rot(sprite, &(&pos + &Vec2I::new(-8, -8)), angle, None);

        if let Some(tractor_beam) = &self.tractor_beam {
            tractor_beam.draw(renderer);
        }
    }

    fn pos(&self) -> &Vec2I { &self.info.pos }
    fn set_pos(&mut self, pos: &Vec2I) { self.info.pos = *pos; }

    fn is_formation(&self) -> bool { self.state == OwlState::Formation }

    fn can_capture_attack(&self) -> bool { true }
    fn is_captured_fighter(&self) -> bool { false }
    fn formation_index(&self) -> &FormationIndex { &self.info.formation_index }

    fn set_damage(
        &mut self, power: u32, accessor: &mut dyn Accessor, event_queue: &mut EventQueue,
    ) -> DamageResult {
        self.owl_set_damage(power, accessor, event_queue)
    }

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>) {
        self.info.update_troop(add, angle_opt);
    }

    fn set_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        self.base.count = 0;
        self.base.attack_frame_count = 0;
        self.copy_angle_to_troops = true;

        for slot in self.troops.iter_mut() {
            *slot = None;
        }
        let phase = if !capture_attack {
            self.capturing_state = CapturingState::None;
            self.copy_angle_to_troops = true;
            self.choose_troops(accessor);

            let flip_x = self.info.formation_index.0 >= 5;
            let mut traj = Traj::new(&OWL_ATTACK_TABLE, &ZERO_VEC, flip_x, self.info.formation_index);
            traj.set_pos(&self.info.pos);

            self.base.traj = Some(traj);
            OwlAttackPhase::Traj
        } else {
            self.capturing_state = CapturingState::Attacking;

            const DLIMIT: i32 = 4 * ONE;
            self.info.speed = 3 * ONE / 2;
            self.info.angle = 0;
            if self.info.formation_index.0 < 5 {
                self.info.vangle = -DLIMIT;
            } else {
                self.info.vangle = DLIMIT;
            }

            let player_pos = accessor.get_player_pos();
            self.base.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

            OwlAttackPhase::Capture
        };

        self.set_state(OwlState::Attack(phase));

        event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
    }

    fn set_to_troop(&mut self) {
        panic!("Illegal");
    }
    fn set_to_formation(&mut self) {
        self.base.set_to_formation(&mut self.info);
        self.set_state(OwlState::Formation);
        if self.life == 0 {
            self.base.disappeared = true;
        }
    }

    #[cfg(debug_assertions)]
    fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool) {
        self.base.set_table_attack(&mut self.info, traj_command_vec, flip_x);
        self.set_state(OwlState::Attack(OwlAttackPhase::Traj));
    }
}

////////////////////////////////////////////////

pub fn create_enemy(
    enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
    fi: &FormationIndex,
) -> Box<dyn Enemy> {
    match enemy_type {
        EnemyType::Owl => { Box::new(Owl::new(pos, angle, speed, fi)) }
        _ => { Box::new(Zako::new(enemy_type, pos, angle, speed, fi)) }
    }
}

pub fn create_appearance_enemy(
    enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
    fi: &FormationIndex, traj: Traj,
) -> Box<dyn Enemy> {
    match enemy_type {
        EnemyType::Owl => {
            let mut owl = Owl::new(pos, angle, speed, fi);
            owl.base.traj = Some(traj);
            owl.set_state(OwlState::Appearance);
            Box::new(owl)
        }
        _ => {
            let mut zako = Zako::new(enemy_type, pos, angle, speed, fi);
            zako.base.traj = Some(traj);
            zako.set_state(ZakoState::Appearance);
            Box::new(zako)
        }
    }
}

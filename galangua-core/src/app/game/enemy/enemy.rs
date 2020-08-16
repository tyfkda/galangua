use super::tractor_beam::TractorBeam;
use super::traj::Traj;
use super::traj_command_table::{BEE_ATTACK_TABLE, BUTTERFLY_ATTACK_TABLE, OWL_ATTACK_TABLE};
use super::{Accessor, FormationIndex};

use crate::app::consts::*;
use crate::app::game::{EventQueue, EventType};
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::framework::RendererTrait;
use crate::util::math::{
    atan2_lut, calc_velocity, clamp, diff_angle, quantize_angle, round_up, square,
    ANGLE, ONE, ONE_BIT};

#[cfg(debug_assertions)]
use super::traj_command::TrajCommand;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
    CapturedFighter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyState {
    None,
    Trajectory,
    MoveToFormation,
    Formation,
    Attack,
    Troop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureState {
    None,
    BeamTracting,
    BeamClosing,
    Capturing,
}

#[derive(Debug)]
pub struct DamageResult {
    pub destroyed: bool,
    pub killed: bool,
    pub point: u32,
}

const MAX_TROOPS: usize = 3;

pub struct Enemy {
    vtable: &'static EnemyVtable,
    pub(super) enemy_type: EnemyType,
    state: EnemyState,
    pos: Vec2I,
    angle: i32,
    speed: i32,
    vangle: i32,
    pub formation_index: FormationIndex,

    life: u32,
    traj: Option<Traj>,
    update_fn: fn(enemy: &mut Enemy, accessor: &mut dyn Accessor, event_queue: &mut EventQueue),
    attack_step: i32,
    count: i32,
    target_pos: Vec2I,
    tractor_beam: Option<TractorBeam>,
    capture_state: CaptureState,
    troops: [Option<FormationIndex>; MAX_TROOPS],
    copy_angle_to_troops: bool,
    disappeared: bool,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32) -> Self {
        let vtable = &ENEMY_VTABLE[enemy_type as usize];

        Self {
            vtable,
            enemy_type,
            state: EnemyState::None,
            life: vtable.life,
            pos: *pos,
            angle,
            speed,
            vangle: 0,
            formation_index: FormationIndex(255, 255),  // Dummy
            traj: None,
            update_fn: update_none,
            attack_step: 0,
            count: 0,
            target_pos: ZERO_VEC,
            tractor_beam: None,
            capture_state: CaptureState::None,
            troops: Default::default(),
            copy_angle_to_troops: true,
            disappeared: false,
        }
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn raw_pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn get_state(&self) -> EnemyState {
        self.state
    }

    pub fn capture_state(&self) -> CaptureState {
        self.capture_state
    }

    pub fn captured_fighter_index(&self) -> Option<FormationIndex> {
        if self.capture_state == CaptureState::Capturing {
            let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
            if self.troops.iter().flat_map(|x| x)
                .find(|index| **index == fi).is_some()
            {
                return Some(fi);
            }
        }
        None
    }

    pub fn is_disappeared(&self) -> bool {
        self.disappeared
    }

    fn is_ghost(&self) -> bool {
        self.life == 0
    }

    pub fn update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        let prev_pos = self.pos;

        (self.update_fn)(self, accessor, event_queue);

        self.pos += calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;

        let angle_opt = if self.copy_angle_to_troops { Some(self.angle) } else { None };
        self.update_troops(&(&self.pos - &prev_pos), angle_opt, accessor);

        if let Some(tractor_beam) = &mut self.tractor_beam {
            tractor_beam.update();
        }

        if self.is_ghost() && !self.disappeared && !self.live_troops(accessor) {
            self.disappeared = true;
        }
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

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>) -> bool {
        self.pos += *add;
        if let Some(angle) = angle_opt {
            self.angle = angle;
        }
        true
    }

    fn release_troops(&mut self, accessor: &mut dyn Accessor) {
        for troop_opt in self.troops.iter_mut() {
            if let Some(index) = troop_opt {
                if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                    enemy.set_to_formation();
                }
                *troop_opt = None;
            }
        }
    }

    pub fn draw<R>(&self, renderer: &mut R, pat: usize) -> Result<(), String>
    where
        R: RendererTrait,
    {
        if self.is_ghost() {
            return Ok(());
        }

        let sprite = (self.vtable.sprite_name)(self, pat);
        let angle = quantize_angle(self.angle, 16);
        let pos = self.pos();
        renderer.draw_sprite_rot(sprite, &(&pos + &Vec2I::new(-8, -8)), angle, None)?;

        if let Some(tractor_beam) = &self.tractor_beam {
            tractor_beam.draw(renderer)?;
        }

        Ok(())
    }

    pub fn set_damage(&mut self, power: u32, accessor: &dyn Accessor,
                      event_queue: &mut EventQueue) -> DamageResult
    {
        (self.vtable.set_damage)(self, power, accessor, event_queue)
    }

    fn live_troops(&self, accessor: &dyn Accessor) -> bool {
        self.troops.iter().flat_map(|x| x)
            .filter_map(|index| accessor.get_enemy_at(index))
            .any(|enemy| enemy.enemy_type != EnemyType::CapturedFighter)
    }

    fn set_state(&mut self, state: EnemyState) {
        let update_fn = match state {
            EnemyState::None | EnemyState::Troop => update_none,
            EnemyState::Trajectory => update_trajectory,
            EnemyState::MoveToFormation => update_move_to_formation,
            EnemyState::Formation => update_formation,
            EnemyState::Attack => update_attack_normal,
        };
        self.set_state_with_fn(state, update_fn);
    }

    fn set_state_with_fn(
        &mut self, state: EnemyState,
        update_fn: fn(enemy: &mut Enemy, accessor: &mut dyn Accessor, event_queue: &mut EventQueue),
    ) {
        self.state = state;
        self.update_fn = update_fn;
    }

    pub fn set_traj(&mut self, traj: Traj) {
        self.traj = Some(traj);
        self.set_state(EnemyState::Trajectory);
    }

    fn update_move_to_formation(&mut self, accessor: &dyn Accessor) -> bool {
        let target = accessor.get_formation_pos(&self.formation_index);
        let diff = &target - &self.pos;
        let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
        if sq_distance > square(self.speed >> (ONE_BIT / 2)) {
            const DLIMIT: i32 = 5 * ONE;
            let target_angle = atan2_lut(-diff.y, diff.x);
            let d = diff_angle(target_angle, self.angle);
            self.angle += clamp(d, -DLIMIT, DLIMIT);
            self.vangle = 0;
            true
        } else {
            self.pos = target;
            self.speed = 0;
            self.angle = 0;
            false
        }
    }

    pub fn set_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor) {
        (self.vtable.set_attack)(self, capture_attack, accessor);
    }

    #[cfg(debug_assertions)]
    pub fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool) {
        let mut traj = Traj::new_with_vec(traj_command_vec, &ZERO_VEC, flip_x, self.formation_index);
        traj.set_pos(&self.pos);

        self.attack_step = 0;
        self.count = 0;
        self.traj = Some(traj);
        self.set_state_with_fn(EnemyState::Attack, update_attack_traj);
    }

    fn choose_troops(&mut self, accessor: &mut dyn Accessor) {
        let base = &self.formation_index;
        let indices = [
            FormationIndex(base.0 - 1, base.1 + 1),
            FormationIndex(base.0 + 1, base.1 + 1),
            FormationIndex(base.0, base.1 - 1),
        ];
        for index in indices.iter() {
            if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                if enemy.state == EnemyState::Formation {
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

    fn add_troop(&mut self, formation_index: FormationIndex) -> bool {
        if let Some(slot) = self.troops.iter_mut().find(|x| x.is_none()) {
            *slot = Some(formation_index);
            true
        } else {
            false
        }
    }

    pub fn set_to_troop(&mut self) {
        self.set_state(EnemyState::Troop);
    }

    pub(super) fn set_to_formation(&mut self) {
        self.speed = 0;
        self.angle = 0;
        self.vangle = 0;
        self.copy_angle_to_troops = true;

        if self.is_ghost() {
            self.disappeared = true;
        }

        self.set_state(EnemyState::Formation);
    }

    fn warp(&mut self, offset: Vec2I) {
        self.pos += offset;
        // No need to modify troops, because offset is calculated from previous position.
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> Option<CollBox> {
        if !self.is_ghost() {
            Some(CollBox {
                top_left: &self.pos() - &Vec2I::new(8, 8),
                size: Vec2I::new(12, 12),
            })
        } else {
            None
        }
    }
}

////////////////////////////////////////////////

struct EnemyVtable {
    life: u32,
    set_attack: fn(me: &mut Enemy, capture_attack: bool, accessor: &mut dyn Accessor),
    calc_point: fn(me: &Enemy) -> u32,
    sprite_name: fn(me: &Enemy, pat: usize) -> &str,
    set_damage: fn(me: &mut Enemy, power: u32, accessor: &dyn Accessor,
                   event_queue: &mut EventQueue) -> DamageResult,
}

fn bee_set_attack(me: &mut Enemy, _capture_attack: bool, _accessor: &mut dyn Accessor) {
    let flip_x = me.formation_index.0 >= 5;
    let mut traj = Traj::new(&BEE_ATTACK_TABLE, &ZERO_VEC, flip_x, me.formation_index);
    traj.set_pos(&me.pos);

    me.attack_step = 0;
    me.count = 0;
    me.traj = Some(traj);
    me.set_state_with_fn(EnemyState::Attack, update_attack_traj);
}

fn butterfly_set_attack(me: &mut Enemy, _capture_attack: bool, _accessor: &mut dyn Accessor) {
    let flip_x = me.formation_index.0 >= 5;
    let mut traj = Traj::new(&BUTTERFLY_ATTACK_TABLE, &ZERO_VEC, flip_x, me.formation_index);
    traj.set_pos(&me.pos);

    me.attack_step = 0;
    me.count = 0;
    me.traj = Some(traj);
    me.set_state_with_fn(EnemyState::Attack, update_attack_traj);
}

fn bee_set_damage(me: &mut Enemy, power: u32, _accessor: &dyn Accessor,
                  _event_queue: &mut EventQueue) -> DamageResult {
    if me.life > power {
        me.life -= power;
        DamageResult { destroyed: false, killed: false, point: 0 }
    } else {
        me.life = 0;
        let point = (me.vtable.calc_point)(me);
        DamageResult { destroyed: true, killed: true, point }
    }
}

const BEE_SPRITE_NAMES: [&str; 2] = ["gopher1", "gopher2"];
const BUTTERFLY_SPRITE_NAMES: [&str; 2] = ["dman1", "dman2"];
const OWL_SPRITE_NAMES: [&str; 4] = ["cpp11", "cpp12", "cpp21", "cpp22"];

const ENEMY_VTABLE: [EnemyVtable; 4] = [
    // Bee
    EnemyVtable {
        life: 1,
        set_attack: bee_set_attack,
        calc_point: |me: &Enemy| {
            if me.state == EnemyState::Formation { 50 } else { 100 }
        },
        sprite_name: |_me: &Enemy, pat: usize| BEE_SPRITE_NAMES[pat],
        set_damage: bee_set_damage,
    },
    // Butterfly
    EnemyVtable {
        life: 1,
        set_attack: butterfly_set_attack,
        calc_point: |me: &Enemy| {
            if me.state == EnemyState::Formation { 80 } else { 160 }
        },
        sprite_name: |_me: &Enemy, pat: usize| BUTTERFLY_SPRITE_NAMES[pat],
        set_damage: bee_set_damage,
    },
    // Owl
    EnemyVtable {
        life: 2,
        set_attack: |me: &mut Enemy, capture_attack: bool, accessor: &mut dyn Accessor| {
            me.attack_step = 0;
            me.count = 0;

            for slot in me.troops.iter_mut() {
                *slot = None;
            }
            let update_fn = if !capture_attack {
                me.copy_angle_to_troops = true;
                me.choose_troops(accessor);

                let flip_x = me.formation_index.0 >= 5;
                let mut traj = Traj::new(&OWL_ATTACK_TABLE, &ZERO_VEC, flip_x, me.formation_index);
                traj.set_pos(&me.pos);

                me.attack_step = 0;
                me.count = 0;
                me.traj = Some(traj);
                update_attack_traj
            } else {
                update_attack_capture
            };

            me.set_state_with_fn(EnemyState::Attack, update_fn);
        },
        calc_point: |me: &Enemy| {
            if me.state == EnemyState::Formation {
                150
            } else {
                let fi = FormationIndex(me.formation_index.0, me.formation_index.1 - 1);
                let count = me.troops.iter().flat_map(|x| x)
                    .filter(|index| **index != fi)
                    .count();
                match count {
                    0 => 400,
                    1 => 800,
                    2 | _ => 1600,
                }
            }
        },
        sprite_name: |me: &Enemy, pat: usize| {
            let pat = if me.life <= 1 { pat + 2 } else { pat };
            OWL_SPRITE_NAMES[pat as usize]
        },
        set_damage: |me: &mut Enemy, power: u32, accessor: &dyn Accessor, _event_queue: &mut EventQueue| -> DamageResult {
            if me.life > power {
                me.life -= power;
                DamageResult { destroyed: false, killed: false, point: 0 }
            } else {
                let mut killed = true;
                me.life = 0;
                if me.live_troops(accessor) {
                    killed = false;  // Keep alive as a ghost.
                }
                let point = (me.vtable.calc_point)(me);
                DamageResult { destroyed: true, killed, point }
            }
        },
    },
    // CapturedFighter
    EnemyVtable {
        life: 1,
        set_attack: bee_set_attack,
        calc_point: |_me: &Enemy| 1000,
        sprite_name: |_me: &Enemy, _pat: usize| "rustacean_captured",
        set_damage: |me: &mut Enemy, power: u32, _accessor: &dyn Accessor, event_queue: &mut EventQueue| -> DamageResult {
            if me.life > power {
                me.life -= power;
                DamageResult { destroyed: false, killed: false, point: 0 }
            } else {
                me.life = 0;
                event_queue.push(EventType::CapturedFighterDestroyed);
                let point = (me.vtable.calc_point)(me);
                DamageResult { destroyed: true, killed: true, point }
            }
        },
    },
];

////////////////////////////////////////////////

fn update_none(_me: &mut Enemy, _accessor: &mut dyn Accessor, _event_queue: &mut EventQueue) {}

fn update_trajectory(me: &mut Enemy, accessor: &mut dyn Accessor, _event_queue: &mut EventQueue) {
    if let Some(traj) = &mut me.traj {
        let cont = traj.update(accessor);

        me.pos = traj.pos();
        me.angle = traj.angle();
        me.speed = traj.speed;
        me.vangle = traj.vangle;

        if !cont {
            me.traj = None;
            me.set_state(EnemyState::MoveToFormation);
        }
    }
}

fn update_move_to_formation(me: &mut Enemy, accessor: &mut dyn Accessor, _event_queue: &mut EventQueue) {
    if !me.update_move_to_formation(accessor) {
        me.release_troops(accessor);
        me.set_to_formation();
    }
}

fn update_formation(me: &mut Enemy, accessor: &mut dyn Accessor, _event_queue: &mut EventQueue) {
    me.pos = accessor.get_formation_pos(&me.formation_index);
}

fn update_attack_normal(me: &mut Enemy, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
    match me.attack_step {
        0 => {
            me.speed = 1 * ONE;
            me.angle = 0;
            if me.formation_index.0 < 5 {
                me.vangle = -4 * ONE;
            } else {
                me.vangle = 4 * ONE;
            }
            me.attack_step += 1;
            me.count = 0;

            event_queue.push(EventType::EneShot(me.pos, 2 * ONE));
        }
        1 => {
            if (me.vangle < 0 && me.angle <= -160 * ONE) ||
                (me.vangle > 0 && me.angle >= 160 * ONE)
            {
                me.vangle = 0;
                me.attack_step += 1;
                me.count = 0;
            }
        }
        2 => {
            me.count += 1;
            if me.count >= 10 {
                if me.formation_index.0 < 5 {
                    me.vangle = 1 * ONE / 4;
                } else {
                    me.vangle = -1 * ONE / 4;
                }
                me.attack_step += 1;
                me.count = 0;
            }
        }
        3 => {
            if (me.vangle > 0 && me.angle >= -ANGLE / 2 * ONE) ||
                (me.vangle < 0 && me.angle <= ANGLE / 2 * ONE)
            {
                me.vangle = 0;
                me.attack_step += 1;
            }
        }
        4 => {
            if me.pos.y >= (HEIGHT + 8) * ONE {
                let target_pos = accessor.get_formation_pos(&me.formation_index);
                let offset = Vec2I::new(target_pos.x - me.pos.x, (-32 - (HEIGHT + 8)) * ONE);
                me.warp(offset);
                me.set_state(EnemyState::MoveToFormation);
            }
        }
        _ => {}
    }
}

fn update_attack_capture(me: &mut Enemy, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
    const DLIMIT: i32 = 4 * ONE;
    match me.attack_step {
        0 => {
            me.speed = 3 * ONE / 2;
            me.angle = 0;
            if me.formation_index.0 < 5 {
                me.vangle = -DLIMIT;
            } else {
                me.vangle = DLIMIT;
            }

            let player_pos = accessor.get_raw_player_pos();
            me.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

            me.attack_step += 1;
            me.count = 0;
        }
        1 => {
            let dpos = &me.target_pos - &me.pos;
            let target_angle = atan2_lut(-dpos.y, dpos.x);
            let mut d = diff_angle(target_angle, me.angle);
            if me.vangle > 0 && d < 0 {
                d += ANGLE * ONE;
            } else if me.vangle < 0 && d > 0 {
                d -= ANGLE * ONE;
            }
            if d >= -DLIMIT && d < DLIMIT {
                me.angle = target_angle;
                me.vangle = 0;
            }

            if me.pos.y >= me.target_pos.y {
                me.pos.y = me.target_pos.y;
                me.speed = 0;
                me.angle = ANGLE / 2 * ONE;
                me.vangle = 0;

                me.tractor_beam = Some(TractorBeam::new(&(&me.pos + &Vec2I::new(0, 8 * ONE))));

                me.attack_step += 1;
                me.count = 0;
            }
        }
        2 => {
            if let Some(tractor_beam) = &mut me.tractor_beam {
                if tractor_beam.closed() {
                    me.tractor_beam = None;
                    me.speed = 5 * ONE / 2;
                    me.attack_step += 1;
                } else if accessor.can_player_capture() &&
                          tractor_beam.can_capture(accessor.get_raw_player_pos()) {
                    event_queue.push(EventType::CapturePlayer(&me.pos + &Vec2I::new(0, 16 * ONE)));
                    tractor_beam.start_capture();
                    me.capture_state = CaptureState::BeamTracting;
                    me.attack_step = 100;
                    me.count = 0;
                }
            }
        }
        3 => {
            if me.pos.y >= (HEIGHT + 8) * ONE {
                let target_pos = accessor.get_formation_pos(&me.formation_index);
                let offset = Vec2I::new(target_pos.x - me.pos.x, (-32 - (HEIGHT + 8)) * ONE);
                me.warp(offset);
                me.set_state(EnemyState::MoveToFormation);
            }
        }
        // Capture sequence
        100 => {
            if accessor.is_player_captured() {
                me.tractor_beam.as_mut().unwrap().close_capture();
                me.capture_state = CaptureState::BeamClosing;
                me.attack_step += 1;
                me.count = 0;
            }
        }
        101 => {
            if let Some(tractor_beam) = &me.tractor_beam {
                if tractor_beam.closed() {
                    let fi = FormationIndex(me.formation_index.0, me.formation_index.1 - 1);
                    event_queue.push(EventType::SpawnCapturedFighter(
                        &me.pos + &Vec2I::new(0, 16 * ONE), fi));

                    me.add_troop(fi);

                    me.tractor_beam = None;
                    me.capture_state = CaptureState::Capturing;
                    event_queue.push(EventType::CapturePlayerCompleted);

                    me.copy_angle_to_troops = false;
                    me.attack_step += 1;
                    me.count = 0;
                }
            }
        }
        102 => {
            me.count += 1;
            if me.count >= 120 {
                me.speed = 3 * ONE / 2;
                me.attack_step += 1;
            }
        }
        103 => {
            if !me.update_move_to_formation(accessor) {
                me.speed = 0;
                me.attack_step += 1;
            }
        }
        104 => {
            let fi = FormationIndex(me.formation_index.0, me.formation_index.1 - 1);
            let mut done = false;
            if let Some(captured_fighter) = accessor.get_enemy_at_mut(&fi) {
                let mut y = captured_fighter.pos.y;
                y -= 1 * ONE;
                let topy = me.pos.y - 16 * ONE;
                if y <= topy {
                    y = topy;
                    done = true;
                }
                captured_fighter.pos.y = y;
            }
            if done {
                event_queue.push(EventType::CaptureSequenceEnded);
                me.release_troops(accessor);
                me.set_to_formation();
            }
        }
        _ => {}
    }
}

fn update_attack_traj(me: &mut Enemy, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
    update_trajectory(me, accessor, event_queue);
}

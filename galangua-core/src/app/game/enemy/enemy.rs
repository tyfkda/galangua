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

const OWL_DESTROY_SHOT_WAIT: u32 = 3 * 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
    CapturedFighter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttackPhase {
    BeeAttack,
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
pub enum EnemyState {
    None,
    Appearance,
    MoveToFormation,
    Assault(u32),
    Formation,
    Attack(AttackPhase),
    Troop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CapturingState {
    None,
    Attacking,
    BeamTracting,
}

#[derive(Debug)]
pub struct DamageResult {
    pub killed: bool,
    pub point: u32,
}

const MAX_TROOPS: usize = 3;

pub struct Enemy {
    vtable: &'static EnemyVtable,
    enemy_type: EnemyType,
    state: EnemyState,
    pos: Vec2I,
    angle: i32,
    speed: i32,
    vangle: i32,
    formation_index: FormationIndex,

    life: u32,
    traj: Option<Traj>,
    shot_wait: Option<u32>,
    count: u32,
    attack_frame_count: u32,
    target_pos: Vec2I,
    tractor_beam: Option<TractorBeam>,
    capturing_state: CapturingState,
    troops: [Option<FormationIndex>; MAX_TROOPS],
    copy_angle_to_troops: bool,
    disappeared: bool,
}

impl Enemy {
    pub fn new(
        enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
        fi: &FormationIndex,
    ) -> Self {
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
            formation_index: *fi,
            traj: None,
            shot_wait: None,
            count: 0,
            attack_frame_count: 0,
            target_pos: ZERO_VEC,
            tractor_beam: None,
            capturing_state: CapturingState::None,
            troops: Default::default(),
            copy_angle_to_troops: true,
            disappeared: false,
        }
    }

    pub fn pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn is_formation(&self) -> bool { self.state == EnemyState::Formation }

    pub fn is_disappeared(&self) -> bool {
        self.disappeared
    }

    fn is_ghost(&self) -> bool {
        self.life == 0
    }

    pub fn can_capture_attack(&self) -> bool { self.enemy_type == EnemyType::Owl }

    pub fn formation_index(&self) -> &FormationIndex { &self.formation_index }

    pub fn update<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        let prev_pos = self.pos;

        self.dispatch_update(accessor, event_queue);

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

    fn update_troops<A: Accessor>(&mut self, add: &Vec2I, angle_opt: Option<i32>, accessor: &mut A) {
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

    fn release_troops<A: Accessor>(&mut self, accessor: &mut A) {
        for troop_opt in self.troops.iter_mut().filter(|x| x.is_some()) {
            let index = &troop_opt.unwrap();
            if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                enemy.set_to_formation();
            }
            *troop_opt = None;
        }
    }

    fn remove_destroyed_troops<A: Accessor>(&mut self, accessor: &mut A) {
        for troop_opt in self.troops.iter_mut().filter(|x| x.is_some()) {
            let index = &troop_opt.unwrap();
            if accessor.get_enemy_at(index).is_none() {
                *troop_opt = None;
            }
        }
    }

    pub fn update_attack<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        self.attack_frame_count += 1;

        let stage_no = accessor.get_stage_no();
        let shot_count = std::cmp::min(2 + stage_no / 8 , 5) as u32;
        let shot_interval = 20 - shot_count * 2;

        if self.attack_frame_count <= shot_interval * shot_count && self.attack_frame_count % shot_interval == 0 {
            event_queue.push(EventType::EneShot(self.pos));
            for troop_fi in self.troops.iter().flat_map(|x| x) {
                if let Some(enemy) = accessor.get_enemy_at(troop_fi) {
                    event_queue.push(EventType::EneShot(enemy.pos));
                }
            }
        }
    }

    pub fn draw<R>(&self, renderer: &mut R, pat: usize)
    where
        R: RendererTrait,
    {
        if self.is_ghost() {
            return;
        }

        let sprite = match self.enemy_type {
            EnemyType::Bee => { BEE_SPRITE_NAMES[pat] }
            EnemyType::Butterfly => { BUTTERFLY_SPRITE_NAMES[pat] }
            EnemyType::Owl => {
                let pat = if self.life <= 1 { pat + 2 } else { pat };
                OWL_SPRITE_NAMES[pat as usize]
            }
            EnemyType::CapturedFighter => { "rustacean_captured" }
        };

        let angle = quantize_angle(self.angle, ANGLE_DIV);
        let pos = round_vec(&self.pos);
        renderer.draw_sprite_rot(sprite, &(&pos + &Vec2I::new(-8, -8)), angle, None);

        if let Some(tractor_beam) = &self.tractor_beam {
            tractor_beam.draw(renderer);
        }
    }

    pub fn set_damage<A: Accessor>(
        &mut self, power: u32, accessor: &mut A, event_queue: &mut EventQueue,
    ) -> DamageResult {
        let result = match self.enemy_type {
            EnemyType::Bee | EnemyType::Butterfly => { self.bee_set_damage(power) }
            EnemyType::Owl => { self.owl_set_damage(power, accessor, event_queue) }
            EnemyType::CapturedFighter => { self.captured_fighter_set_damage(power, event_queue) }
        };

        if result.point > 0 {
            event_queue.push(EventType::EnemyExplosion(self.pos, self.angle, self.enemy_type));
        }
        result
    }

    fn live_troops<A: Accessor>(&self, accessor: &A) -> bool {
        self.troops.iter().flat_map(|x| x)
            .filter_map(|index| accessor.get_enemy_at(index))
            .any(|enemy| enemy.enemy_type != EnemyType::CapturedFighter)
    }

    fn set_state(&mut self, state: EnemyState) {
        self.state = state;
    }

    pub fn set_appearance(&mut self, traj: Traj) {
        self.traj = Some(traj);
        self.set_state(EnemyState::Appearance);
    }

    fn move_to_formation<A: Accessor>(&mut self, accessor: &A) -> bool {
        let target = accessor.get_formation_pos(&self.formation_index);
        let diff = &target - &self.pos;
        let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
        if sq_distance > square(self.speed >> (ONE_BIT / 2)) {
            let dlimit: i32 = self.speed * 5 / 3;
            let target_angle = atan2_lut(-diff.y, diff.x);
            let d = diff_angle(target_angle, self.angle);
            self.angle += clamp(d, -dlimit, dlimit);
            self.vangle = 0;
            true
        } else {
            self.pos = target;
            self.speed = 0;
            self.capturing_state = CapturingState::None;
            false
        }
    }

    pub fn set_attack<A: Accessor>(&mut self, capture_attack: bool, accessor: &mut A, event_queue: &mut EventQueue) {
        match self.enemy_type {
            EnemyType::Bee => { self.set_bee_attack(); }
            EnemyType::Butterfly => { self.set_butterfly_attack(); }
            EnemyType::Owl => { self.set_owl_attack(capture_attack, accessor); }
            EnemyType::CapturedFighter => { self.set_captured_fighter_attack(); }
        }

        event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
    }

    #[cfg(debug_assertions)]
    pub fn set_pos(&mut self, pos: &Vec2I) {
        self.pos = *pos;
    }

    #[cfg(debug_assertions)]
    pub fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool) {
        let mut traj = Traj::new_with_vec(traj_command_vec, &ZERO_VEC, flip_x, self.formation_index);
        traj.set_pos(&self.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
        self.set_state(EnemyState::Attack(AttackPhase::Traj));
    }

    fn choose_troops<A: Accessor>(&mut self, accessor: &mut A) {
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
        self.angle = normalize_angle(self.angle);
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

    fn rush_attack(&mut self) {
        let flip_x = self.formation_index.0 >= 5;
        let table = self.vtable.rush_traj_table;
        let mut traj = Traj::new(table, &ZERO_VEC, flip_x, self.formation_index);
        traj.set_pos(&self.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);

        self.set_state(EnemyState::Attack(AttackPhase::Traj));
    }

    //// set_attack

    fn set_bee_attack(&mut self) {
        let flip_x = self.formation_index.0 >= 5;
        let mut traj = Traj::new(&BEE_ATTACK_TABLE, &ZERO_VEC, flip_x, self.formation_index);
        traj.set_pos(&self.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
        self.set_state(EnemyState::Attack(AttackPhase::BeeAttack));
    }

    fn set_butterfly_attack(&mut self) {
        let flip_x = self.formation_index.0 >= 5;
        let mut traj = Traj::new(&BUTTERFLY_ATTACK_TABLE, &ZERO_VEC, flip_x, self.formation_index);
        traj.set_pos(&self.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
        self.set_state(EnemyState::Attack(AttackPhase::Traj));
    }

    fn set_captured_fighter_attack(&mut self) {
        let flip_x = self.formation_index.0 >= 5;
        let mut traj = Traj::new(&OWL_ATTACK_TABLE, &ZERO_VEC, flip_x, self.formation_index);
        traj.set_pos(&self.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
        self.set_state(EnemyState::Attack(AttackPhase::Traj));
    }

    fn set_owl_attack<A: Accessor>(&mut self, capture_attack: bool, accessor: &mut A) {
        self.count = 0;
        self.attack_frame_count = 0;

        for slot in self.troops.iter_mut() {
            *slot = None;
        }
        let phase = if !capture_attack {
            self.copy_angle_to_troops = true;
            self.choose_troops(accessor);

            let flip_x = self.formation_index.0 >= 5;
            let mut traj = Traj::new(&OWL_ATTACK_TABLE, &ZERO_VEC, flip_x, self.formation_index);
            traj.set_pos(&self.pos);

            self.traj = Some(traj);
            AttackPhase::Traj
        } else {
            self.capturing_state = CapturingState::Attacking;

            const DLIMIT: i32 = 4 * ONE;
            self.speed = 3 * ONE / 2;
            self.angle = 0;
            if self.formation_index.0 < 5 {
                self.vangle = -DLIMIT;
            } else {
                self.vangle = DLIMIT;
            }

            let player_pos = accessor.get_player_pos();
            self.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

            AttackPhase::Capture
        };

        self.set_state(EnemyState::Attack(phase));
    }

    //// set_damage

    fn bee_set_damage(&mut self, power: u32) -> DamageResult {
        if self.life > power {
            self.life -= power;
            DamageResult { killed: false, point: 0 }
        } else {
            self.life = 0;
            let point = self.calc_point();
            DamageResult { killed: true, point }
        }
    }

    fn owl_set_damage<A: Accessor>(
        &mut self, power: u32, accessor: &mut A, event_queue: &mut EventQueue,
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
                    let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
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

            DamageResult { killed, point }
        }
    }

    fn captured_fighter_set_damage(&mut self, power: u32, event_queue: &mut EventQueue) -> DamageResult {
        if self.life > power {
            self.life -= power;
            DamageResult { killed: false, point: 0 }
        } else {
            self.life = 0;
            event_queue.push(EventType::CapturedFighterDestroyed);
            let point = self.calc_point();
            DamageResult { killed: true, point }
        }
    }

    fn calc_point(&self) -> u32 {
        match self.enemy_type {
            EnemyType::Bee => {
                if self.state == EnemyState::Formation { 50 } else { 100 }
            }
            EnemyType::Butterfly => {
                if self.state == EnemyState::Formation { 80 } else { 160 }
            }
            EnemyType::Owl => {
                if self.state == EnemyState::Formation {
                    150
                } else {
                    let cap_fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
                    let count = self.troops.iter().flat_map(|x| x)
                        .filter(|index| **index != cap_fi)
                        .count();
                    (1 << count) * 400
                }
            }
            EnemyType::CapturedFighter => {
                if self.state == EnemyState::Formation { 500 } else { 1000 }
            }
        }
    }

    //// Update

    fn dispatch_update<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        match self.state {
            EnemyState::None | EnemyState::Troop => {}
            EnemyState::Appearance => { self.update_trajectory(accessor, event_queue); }
            EnemyState::MoveToFormation => { self.update_move_to_formation(accessor); }
            EnemyState::Assault(phase) => { self.update_assault(phase); }
            EnemyState::Formation => { self.update_formation(accessor); }
            EnemyState::Attack(phase) => {
                match phase {
                    AttackPhase::BeeAttack => { self.update_bee_attack(accessor, event_queue) }
                    AttackPhase::Traj => { self.update_attack_traj(accessor, event_queue); }
                    AttackPhase::Capture => { self.update_attack_capture(); }
                    AttackPhase::CaptureBeam => { self.update_attack_capture_beam(accessor, event_queue); }
                    AttackPhase::NoCaptureGoOut => { self.update_attack_capture_go_out(accessor, event_queue); }
                    AttackPhase::CaptureStart => { self.update_attack_capture_start(accessor); }
                    AttackPhase::CaptureCloseBeam => { self.update_attack_capture_close_beam(event_queue); }
                    AttackPhase::CaptureDoneWait => { self.update_attack_capture_done_wait(); }
                    AttackPhase::CaptureDoneBack => { self.update_attack_capture_back(accessor); }
                    AttackPhase::CaptureDonePushUp => { self.update_attack_capture_push_up(accessor, event_queue); }
                }
            }
        }
    }

    fn update_trajectory<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        if let Some(traj) = &mut self.traj {
            let cont = traj.update(accessor);

            self.pos = traj.pos();
            self.angle = traj.angle();
            self.speed = traj.speed;
            self.vangle = traj.vangle;
            if let Some(wait) = traj.is_shot() {
                self.shot_wait = Some(wait);
            }

            if let Some(wait) = self.shot_wait {
                if wait > 0 {
                    self.shot_wait = Some(wait - 1);
                } else {
                    event_queue.push(EventType::EneShot(self.pos));
                    self.shot_wait = None;
                }
            }

            if cont {
                return;
            }
        }

        self.traj = None;

        if self.state == EnemyState::Appearance &&
            self.formation_index.1 >= Y_COUNT as u8  // Assault
        {
            let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
            let target_pos = [
                Some(*accessor.get_player_pos()),
                accessor.get_dual_player_pos(),
            ];
            let count = target_pos.iter().flat_map(|x| x).count();
            let target: &Vec2I = target_pos.iter()
                .flat_map(|x| x).nth(rng.gen_range(0, count)).unwrap();

            self.target_pos = *target;
            self.vangle = 0;
            self.set_state(EnemyState::Assault(0));
        } else {
            self.set_state(EnemyState::MoveToFormation);
        }
    }

    fn update_move_to_formation<A: Accessor>(&mut self, accessor: &mut A) {
        if !self.move_to_formation(accessor) {
            self.release_troops(accessor);
            self.set_to_formation();
        }
    }

    fn update_assault(&mut self, phase: u32) {
        match phase {
            0 => {
                let target = &self.target_pos;
                let diff = target - &self.pos;

                const DLIMIT: i32 = 5 * ONE;
                let target_angle = atan2_lut(-diff.y, diff.x);
                let d = diff_angle(target_angle, self.angle);
                if d < -DLIMIT {
                    self.angle -= DLIMIT;
                } else if d > DLIMIT {
                    self.angle += DLIMIT;
                } else {
                    self.angle += d;
                    self.set_state(EnemyState::Assault(1));
                }
            }
            1 | _ => {
                if self.pos.y >= (HEIGHT + 8) * ONE {
                    self.disappeared = true;
                }
            }
        }
    }

    fn update_formation<A: Accessor>(&mut self, accessor: &mut A) {
        self.pos = accessor.get_formation_pos(&self.formation_index);

        let ang = ANGLE * ONE / 128;
        self.angle -= clamp(self.angle, -ang, ang);
    }

    fn update_bee_attack<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        self.update_attack(accessor, event_queue);
        self.update_trajectory(accessor, event_queue);

        if let EnemyState::Attack(_) = self.state {
            if accessor.is_rush() {
                let flip_x = self.formation_index.0 >= 5;
                let mut traj = Traj::new(&BEE_ATTACK_RUSH_CONT_TABLE, &ZERO_VEC, flip_x, self.formation_index);
                traj.set_pos(&self.pos);

                self.traj = Some(traj);
                self.set_state(EnemyState::Attack(AttackPhase::Traj));

                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
            }
        }
    }

    fn update_attack_capture(&mut self) {
        const DLIMIT: i32 = 4 * ONE;
        let dpos = &self.target_pos - &self.pos;
        let target_angle = atan2_lut(-dpos.y, dpos.x);
        let ang_limit = ANGLE * ONE / 2 - ANGLE * ONE * 30 / 360;
        let target_angle = if target_angle >= 0 {
            std::cmp::max(target_angle, ang_limit)
        } else {
            std::cmp::min(target_angle, -ang_limit)
        };
        let mut d = diff_angle(target_angle, self.angle);
        if self.vangle > 0 && d < 0 {
            d += ANGLE * ONE;
        } else if self.vangle < 0 && d > 0 {
            d -= ANGLE * ONE;
        }
        if d >= -DLIMIT && d < DLIMIT {
            self.angle = target_angle;
            self.vangle = 0;
        }

        if self.pos.y >= self.target_pos.y {
            self.pos.y = self.target_pos.y;
            self.speed = 0;
            self.angle = ANGLE / 2 * ONE;
            self.vangle = 0;

            self.tractor_beam = Some(TractorBeam::new(&(&self.pos + &Vec2I::new(0, 8 * ONE))));

            self.set_state(EnemyState::Attack(AttackPhase::CaptureBeam));
            self.count = 0;
        }
    }
    fn update_attack_capture_beam<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        if let Some(tractor_beam) = &mut self.tractor_beam {
            if tractor_beam.closed() {
                self.tractor_beam = None;
                self.speed = 5 * ONE / 2;
                self.set_state(EnemyState::Attack(AttackPhase::NoCaptureGoOut));
            } else if accessor.can_player_capture() &&
                      tractor_beam.can_capture(accessor.get_player_pos())
            {
                event_queue.push(EventType::CapturePlayer(&self.pos + &Vec2I::new(0, 16 * ONE)));
                tractor_beam.start_capture();
                self.capturing_state = CapturingState::BeamTracting;
                self.set_state(EnemyState::Attack(AttackPhase::CaptureStart));
                self.count = 0;
            }
        }
    }
    fn update_attack_capture_go_out<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        if self.pos.y >= (HEIGHT + 8) * ONE {
            let target_pos = accessor.get_formation_pos(&self.formation_index);
            let offset = Vec2I::new(target_pos.x - self.pos.x, (-32 - (HEIGHT + 8)) * ONE);
            self.warp(offset);

            if accessor.is_rush() {
                self.rush_attack();
                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
            } else {
                self.set_state(EnemyState::MoveToFormation);
                self.capturing_state = CapturingState::None;
                event_queue.push(EventType::EndCaptureAttack);
            }
        }
    }
    fn update_attack_capture_start<A: Accessor>(&mut self, accessor: &mut A) {
        if accessor.is_player_capture_completed() {
            self.tractor_beam.as_mut().unwrap().close_capture();
            self.set_state(EnemyState::Attack(AttackPhase::CaptureCloseBeam));
            self.count = 0;
        }
    }
    fn update_attack_capture_close_beam(&mut self, event_queue: &mut EventQueue) {
        if let Some(tractor_beam) = &self.tractor_beam {
            if tractor_beam.closed() {
                let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
                event_queue.push(EventType::SpawnCapturedFighter(
                    &self.pos + &Vec2I::new(0, 16 * ONE), fi));

                self.add_troop(fi);

                self.tractor_beam = None;
                self.capturing_state = CapturingState::Attacking;
                event_queue.push(EventType::CapturePlayerCompleted);

                self.copy_angle_to_troops = false;
                self.set_state(EnemyState::Attack(AttackPhase::CaptureDoneWait));
                self.count = 0;
            }
        }
    }
    fn update_attack_capture_done_wait(&mut self) {
        self.count += 1;
        if self.count >= 120 {
            self.speed = 5 * ONE / 2;
            self.set_state(EnemyState::Attack(AttackPhase::CaptureDoneBack));
        }
    }
    fn update_attack_capture_back<A: Accessor>(&mut self, accessor: &mut A) {
        if !self.move_to_formation(accessor) {
            self.speed = 0;
            self.angle = normalize_angle(self.angle);
            self.set_state(EnemyState::Attack(AttackPhase::CaptureDonePushUp));
        }
    }
    fn update_attack_capture_push_up<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        let ang = ANGLE * ONE / 128;
        self.angle -= clamp(self.angle, -ang, ang);

        let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
        let mut done = false;
        if let Some(captured_fighter) = accessor.get_enemy_at_mut(&fi) {
            let mut y = captured_fighter.pos.y;
            y -= 1 * ONE;
            let topy = self.pos.y - 16 * ONE;
            if y <= topy {
                y = topy;
                done = true;
            }
            captured_fighter.pos.y = y;
        }
        if done {
            event_queue.push(EventType::CaptureSequenceEnded);
            self.release_troops(accessor);
            self.set_to_formation();
        }
    }

    fn update_attack_traj<A: Accessor>(&mut self, accessor: &mut A, event_queue: &mut EventQueue) {
        self.update_attack(accessor, event_queue);
        self.update_trajectory(accessor, event_queue);

        if let EnemyState::Attack(_) = self.state {
        } else {
            if self.enemy_type == EnemyType::CapturedFighter {
                self.disappeared = true;
            } else if accessor.is_rush() {
                // Rush mode: Continue attacking
                self.remove_destroyed_troops(accessor);
                self.rush_attack();
                event_queue.push(EventType::PlaySe(CH_JINGLE, SE_ATTACK_START));
            }
        }
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> Option<CollBox> {
        if !self.is_ghost() {
            Some(CollBox {
                top_left: &round_vec(&self.pos) - &Vec2I::new(6, 6),
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
    rush_traj_table: &'static [TrajCommand],
}

const BEE_SPRITE_NAMES: [&str; 2] = ["gopher1", "gopher2"];
const BUTTERFLY_SPRITE_NAMES: [&str; 2] = ["dman1", "dman2"];
const OWL_SPRITE_NAMES: [&str; 4] = ["cpp11", "cpp12", "cpp21", "cpp22"];

const ENEMY_VTABLE: [EnemyVtable; 4] = [
    // Bee
    EnemyVtable {
        life: 1,
        rush_traj_table: &BEE_RUSH_ATTACK_TABLE,
    },
    // Butterfly
    EnemyVtable {
        life: 1,
        rush_traj_table: &BUTTERFLY_RUSH_ATTACK_TABLE,
    },
    // Owl
    EnemyVtable {
        life: 2,
        rush_traj_table: &OWL_RUSH_ATTACK_TABLE,
    },
    // CapturedFighter
    EnemyVtable {
        life: 1,
        rush_traj_table: &OWL_RUSH_ATTACK_TABLE,
    },
];

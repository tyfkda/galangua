use super::enemy::Enemy;
use super::enemy_base::{EnemyBase, EnemyInfo};
use super::tractor_beam::TractorBeam;
use super::traj::Traj;
use super::traj_command_table::*;
use super::{Accessor, DamageResult, EnemyType, FormationIndex};

use crate::app::consts::*;
use crate::app::game::manager::formation::Y_COUNT;
use crate::app::game::manager::EventType;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::framework::RendererTrait;
use crate::util::math::{
    atan2_lut, clamp, diff_angle, normalize_angle, quantize_angle, round_vec,
    ANGLE, ONE};

#[cfg(debug_assertions)]
use super::traj_command::TrajCommand;

const MAX_TROOPS: usize = 3;
const OWL_DESTROY_SHOT_WAIT: u32 = 3 * 60;

const OWL_SPRITE_NAMES: [&str; 4] = ["cpp11", "cpp12", "cpp21", "cpp22"];

#[derive(Clone, Copy, PartialEq)]
pub(super) enum OwlAttackPhase {
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

#[derive(Clone, Copy, PartialEq)]
pub(super) enum OwlState {
    None,
    Appearance,
    MoveToFormation,
    Assault(u32),
    Formation,
    Attack(OwlAttackPhase),
}

#[derive(Clone, Copy, PartialEq)]
enum CapturingState {
    None,
    Attacking,
    BeamTracting,
}

pub struct Owl {
    pub(super) info: EnemyInfo,
    pub(super) base: EnemyBase,
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

    pub(super) fn set_state(&mut self, state: OwlState) {
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

    fn dispatch_update(&mut self, accessor: &mut dyn Accessor) {
        match self.state {
            OwlState::None => {}
            OwlState::Appearance => {
                if !self.base.update_trajectory(&mut self.info, accessor) {
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
                    OwlAttackPhase::Traj => { self.update_attack_traj(accessor); }
                    OwlAttackPhase::Capture => { self.update_attack_capture(); }
                    OwlAttackPhase::CaptureBeam => { self.update_attack_capture_beam(accessor); }
                    OwlAttackPhase::NoCaptureGoOut => { self.update_attack_capture_go_out(accessor); }
                    OwlAttackPhase::CaptureStart => { self.update_attack_capture_start(accessor); }
                    OwlAttackPhase::CaptureCloseBeam => { self.update_attack_capture_close_beam(accessor); }
                    OwlAttackPhase::CaptureDoneWait => { self.update_attack_capture_done_wait(); }
                    OwlAttackPhase::CaptureDoneBack => { self.update_attack_capture_back(accessor); }
                    OwlAttackPhase::CaptureDonePushUp => { self.update_attack_capture_push_up(accessor); }
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
    fn update_attack_capture_beam(&mut self, accessor: &mut dyn Accessor) {
        if let Some(tractor_beam) = &mut self.tractor_beam {
            if tractor_beam.closed() {
                self.tractor_beam = None;
                self.info.speed = 5 * ONE / 2;
                self.set_state(OwlState::Attack(OwlAttackPhase::NoCaptureGoOut));
            } else if accessor.can_player_capture() &&
                      tractor_beam.can_capture(accessor.get_player_pos())
            {
                accessor.push_event(EventType::CapturePlayer(&self.info.pos + &Vec2I::new(0, 16 * ONE)));
                tractor_beam.start_capture();
                self.capturing_state = CapturingState::BeamTracting;
                self.set_state(OwlState::Attack(OwlAttackPhase::CaptureStart));
                self.base.count = 0;
            }
        }
    }
    fn update_attack_capture_go_out(&mut self, accessor: &mut dyn Accessor) {
        if self.info.pos.y >= (HEIGHT + 8) * ONE {
            let target_pos = accessor.get_formation_pos(&self.info.formation_index);
            let offset = Vec2I::new(target_pos.x - self.info.pos.x, (-32 - (HEIGHT + 8)) * ONE);
            self.info.pos += offset;

            if accessor.is_rush() {
                self.rush_attack();
                accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
            } else {
                self.set_state(OwlState::MoveToFormation);
                self.capturing_state = CapturingState::None;
                accessor.push_event(EventType::EndCaptureAttack);
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
    fn update_attack_capture_close_beam(&mut self, accessor: &mut dyn Accessor) {
        if let Some(tractor_beam) = &self.tractor_beam {
            if tractor_beam.closed() {
                let fi = FormationIndex(self.info.formation_index.0, self.info.formation_index.1 - 1);
                accessor.push_event(EventType::SpawnCapturedFighter(
                    &self.info.pos + &Vec2I::new(0, 16 * ONE), fi));

                self.add_troop(fi);

                self.tractor_beam = None;
                self.capturing_state = CapturingState::None;
                accessor.push_event(EventType::CapturePlayerCompleted);

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
    fn update_attack_capture_push_up(&mut self, accessor: &mut dyn Accessor) {
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
            accessor.push_event(EventType::CaptureSequenceEnded);
            self.release_troops(accessor);
            self.set_to_formation();
        }
    }

    fn update_attack_traj(&mut self, accessor: &mut dyn Accessor) {
        self.update_attack(accessor);
        if !self.base.update_trajectory(&mut self.info, accessor) {
            if accessor.is_rush() {
                // Rush mode: Continue attacking
                self.remove_destroyed_troops(accessor);
                self.rush_attack();
                accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
            } else {
                self.set_state(OwlState::MoveToFormation);
            }
        }
    }

    fn update_attack(&mut self, accessor: &mut dyn Accessor) {
        if self.base.update_attack(&mut self.info, accessor) {
            for troop_fi in self.troops.iter().flat_map(|x| x) {
                let pos_opt = if let Some(troop) = accessor.get_enemy_at(troop_fi) {
                    Some(*troop.pos())
                } else {
                    None
                };
                if let Some(pos) = pos_opt {
                    accessor.push_event(EventType::EneShot(pos));
                }
            }
        }
    }

    fn owl_set_damage(&mut self, power: u32, accessor: &mut dyn Accessor) -> DamageResult {
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
                        accessor.push_event(EventType::RecapturePlayer(fi));
                    }
                }
                CapturingState::Attacking => {
                    accessor.push_event(EventType::EndCaptureAttack);
                }
                CapturingState::BeamTracting => {
                    accessor.push_event(EventType::EscapeCapturing);
                }
            }
            self.capturing_state = CapturingState::None;

            accessor.pause_enemy_shot(OWL_DESTROY_SHOT_WAIT);

            accessor.push_event(EventType::EnemyExplosion(self.info.pos, self.info.angle, EnemyType::Owl));

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
    fn update(&mut self, accessor: &mut dyn Accessor) -> bool {
        let prev_pos = self.info.pos;

        self.dispatch_update(accessor);
        self.info.forward();

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

    fn set_damage(&mut self, power: u32, accessor: &mut dyn Accessor) -> DamageResult {
        self.owl_set_damage(power, accessor)
    }

    fn update_troop(&mut self, _add: &Vec2I, _angle_opt: Option<i32>) {
        panic!("Illegal");
    }

    fn set_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor) {
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

        accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
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
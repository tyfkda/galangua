use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::traj::Accessor as TrajAccessor;
use super::traj::Traj;
use super::traj_command::TrajCommand;
use super::{Accessor, FormationIndex};

use crate::app::consts::*;
use crate::app::game::manager::EventType;
use crate::app::util::collision::CollBox;
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::{
    atan2_lut, calc_velocity, clamp, diff_angle, normalize_angle, round_vec, square, ANGLE, ONE, ONE_BIT,
};

struct TrajAccessorImpl<'a> {
    accessor: &'a dyn Accessor,
}
impl<'a> TrajAccessor for TrajAccessorImpl<'a> {
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.accessor.get_formation_pos(formation_index)
    }
    fn get_stage_no(&self) -> u16 { self.accessor.get_stage_no() }
}

pub struct EnemyInfo {
    pub(super) pos: Vec2I,
    pub(super) angle: i32,
    pub(super) speed: i32,
    pub(super) vangle: i32,
    pub(super) formation_index: FormationIndex,
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

    pub(super) fn forward(&mut self) {
        self.pos += &calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;
    }

    pub(super) fn update_formation(&mut self, accessor: &mut dyn Accessor) {
        self.pos = accessor.get_formation_pos(&self.formation_index);

        let ang = ANGLE * ONE / 128;
        self.angle -= clamp(self.angle, -ang, ang);
    }

    //impl Collidable for EnemyBase
    pub(super) fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: &round_vec(&self.pos) - &Vec2I::new(6, 6),
            size: Vec2I::new(12, 12),
        }
    }
}

pub struct EnemyBase {
    pub(super) traj: Option<Traj>,
    pub(super) shot_wait: Option<u32>,
    pub(super) count: u32,
    pub(super) attack_frame_count: u32,
    pub(super) target_pos: Vec2I,
    pub(super) disappeared: bool,
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

    pub fn update_attack(&mut self, info: &EnemyInfo, accessor: &mut dyn Accessor) -> bool {
        self.attack_frame_count += 1;

        let stage_no = accessor.get_stage_no();
        let shot_count = std::cmp::min(2 + stage_no / 8, 5) as u32;
        let shot_interval = 20 - shot_count * 2;

        if self.attack_frame_count <= shot_interval * shot_count &&
            self.attack_frame_count % shot_interval == 0
        {
            accessor.push_event(EventType::EneShot(info.pos));
            true
        } else {
            false
        }
    }

    pub(super) fn move_to_formation(&mut self, info: &mut EnemyInfo, accessor: &dyn Accessor) -> bool {
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

    pub(super) fn rush_attack(&mut self, info: &EnemyInfo, table: &'static [TrajCommand]) {
        let flip_x = info.formation_index.0 >= 5;
        let mut traj = Traj::new(table, &ZERO_VEC, flip_x, info.formation_index);
        traj.set_pos(&info.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
    }

    pub(super) fn set_assault(&mut self, info: &mut EnemyInfo, accessor: &dyn Accessor) {
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

    pub(super) fn update_trajectory(&mut self, info: &mut EnemyInfo, accessor: &mut dyn Accessor) -> bool {
        if let Some(traj) = &mut self.traj {
            let traj_accessor = TrajAccessorImpl { accessor };
            let cont = traj.update(&traj_accessor);

            info.pos = traj.pos();
            info.angle = traj.angle;
            info.speed = traj.speed;
            info.vangle = traj.vangle;
            if let Some(wait) = traj.is_shot() {
                self.shot_wait = Some(wait);
            }

            if let Some(wait) = self.shot_wait {
                if wait > 0 {
                    self.shot_wait = Some(wait - 1);
                } else {
                    accessor.push_event(EventType::EneShot(info.pos));
                    self.shot_wait = None;
                }
            }

            if cont {
                return true;
            }
            self.traj = None;
        }
        false
    }

    pub(super) fn update_assault(&mut self, info: &mut EnemyInfo, mut phase: u32) -> u32 {
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
    pub(super) fn set_to_formation(&mut self, info: &mut EnemyInfo) {
        info.speed = 0;
        info.angle = normalize_angle(info.angle);
        info.vangle = 0;
    }

    #[cfg(debug_assertions)]
    pub(super) fn set_table_attack(&mut self, info: &mut EnemyInfo, traj_command_vec: Vec<TrajCommand>, flip_x: bool) {
        let mut traj = Traj::new_with_vec(traj_command_vec, &ZERO_VEC, flip_x, info.formation_index);
        traj.set_pos(&info.pos);

        self.count = 0;
        self.attack_frame_count = 0;
        self.traj = Some(traj);
    }
}

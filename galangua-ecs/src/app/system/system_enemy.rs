use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::formation_table::X_COUNT;
use galangua_common::app::game::traj::Accessor as TrajAccessor;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command::TrajCommand;
use galangua_common::app::game::traj_command_table::*;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::util::math::{atan2_lut, calc_velocity, clamp, diff_angle, normalize_angle, square, ANGLE, ONE, ONE_BIT};

use crate::app::components::*;

pub fn forward(posture: &mut Posture, speed: &Speed) {
    posture.0 += &calc_velocity(posture.1 + speed.1 / 2, speed.0);
    posture.1 += speed.1;
}

pub fn move_to_formation(posture: &mut Posture, speed: &mut Speed, fi: &FormationIndex, formation: &Formation) -> bool {
    let target = formation.pos(fi);
    let pos = &mut posture.0;
    let angle = &mut posture.1;
    let spd = &mut speed.0;
    let vangle = &mut speed.1;
    let diff = &target - &pos;
    let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
    if sq_distance > square(*spd >> (ONE_BIT / 2)) {
        let dlimit: i32 = *spd * 5 / 3;
        let target_angle = atan2_lut(-diff.y, diff.x);
        let d = diff_angle(target_angle, *angle);
        *angle += clamp(d, -dlimit, dlimit);
        *vangle = 0;
        false
    } else {
        *pos = target;
        *spd = 0;
        *angle = normalize_angle(*angle);
        *vangle = 0;
        true
    }
}

pub fn update_traj(traj: &mut Traj, posture: &mut Posture, vel: &mut Speed, formation: &Formation) -> bool {
    let traj_accessor = TrajAccessorImpl { formation: &formation };
    let cont = traj.update(&traj_accessor);

    posture.0 = traj.pos();
    posture.1 = traj.angle;
    vel.0 = traj.speed;
    vel.1 = traj.vangle;
    //if let Some(wait) = traj.is_shot() {
    //    self.shot_wait = Some(wait);
    //}
    cont
}

pub fn do_move_zako(zako: &mut Zako, enemy: &mut Enemy, posture: &mut Posture, speed: &mut Speed, formation: &Formation) {
    match zako.state {
        ZakoState::Appearance => {
            let cont = if let Some(traj) = &mut zako.traj.as_mut() {
                update_traj(traj, posture, speed, formation)
            } else {
                false
            };
            if !cont {
                zako.traj = None;
                zako.state = ZakoState::MoveToFormation;
            }
        }
        ZakoState::Formation => {
            posture.0 = formation.pos(&enemy.formation_index);

            let ang = ANGLE * ONE / 128;
            posture.1 -= clamp(posture.1, -ang, ang);
        }
        ZakoState::Attack => {
            let cont = if let Some(traj) = &mut zako.traj.as_mut() {
                update_traj(traj, posture, speed, formation)
            } else {
                false
            };
            if !cont {
                zako.traj = None;
                zako.state = ZakoState::MoveToFormation;
            }
        }
        ZakoState::MoveToFormation => {
            let result = move_to_formation(posture, speed, &enemy.formation_index, formation);
            forward(posture, speed);
            if result {
                zako.state = ZakoState::Formation;
                enemy.is_formation = true;
            }
        }
        ZakoState::Troop => {
            // Controlled by leader.
        }
    }
}

pub fn zako_start_attack(zako: &mut Zako, enemy: &mut Enemy, posture: &Posture) {
    let flip_x = enemy.formation_index.0 >= (X_COUNT as u8) / 2;
    let table: &[TrajCommand] = match enemy.enemy_type {
        EnemyType::Bee => &BEE_ATTACK_TABLE,
        EnemyType::Butterfly => &BUTTERFLY_ATTACK_TABLE,
        EnemyType::Owl => &OWL_ATTACK_TABLE,
        EnemyType::CapturedFighter => &OWL_ATTACK_TABLE,
    };
    let mut traj = Traj::new(table, &ZERO_VEC, flip_x, enemy.formation_index.clone());
    traj.set_pos(&posture.0);
    zako.traj = Some(traj);
    zako.state = ZakoState::Attack;
    enemy.is_formation = false;
}

pub fn set_zako_to_troop(zako: &mut Zako, enemy: &mut Enemy) {
    zako.state = ZakoState::Troop;
    enemy.is_formation = false;
}

//

struct TrajAccessorImpl<'a> {
    formation: &'a Formation,
}
impl<'a> TrajAccessor for TrajAccessorImpl<'a> {
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }
    fn get_stage_no(&self) -> u16 { 0 }
}

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

pub fn do_move_zako(zako: &mut Zako, enemy: &Enemy, posture: &mut Posture, speed: &mut Speed, formation: &Formation) {
    match zako.state {
        ZakoState::Appearance => {
            if !update_traj(zako, posture, speed, formation) {
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
            if !update_traj(zako, posture, speed, formation) {
                zako.traj = None;
                zako.state = ZakoState::MoveToFormation;
            }
        }
        ZakoState::MoveToFormation => {
            let target = formation.pos(&enemy.formation_index);
            let pos = &mut posture.0;
            let angle = &mut posture.1;
            let spd = &mut speed.0;
            let vangle = &mut speed.1;
            let diff = &target - &pos;
            let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
            let cont = if sq_distance > square(*spd >> (ONE_BIT / 2)) {
                let dlimit: i32 = *spd * 5 / 3;
                let target_angle = atan2_lut(-diff.y, diff.x);
                let d = diff_angle(target_angle, *angle);
                *angle += clamp(d, -dlimit, dlimit);
                *vangle = 0;
                *pos += &calc_velocity(*angle, *spd);
                true
            } else {
                *pos = target;
                *spd = 0;
                *angle = normalize_angle(*angle);
                *vangle = 0;
                false
            };
            if !cont {
                zako.state = ZakoState::Formation;
            }
        }
    }
}

pub fn zako_start_attack(zako: &mut Zako, enemy: &Enemy, posture: &Posture, _capture_attack: bool) {
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
}

fn update_traj(zako: &mut Zako, posture: &mut Posture, speed: &mut Speed, formation: &Formation) -> bool {
    if let Some(traj) = &mut zako.traj.as_mut() {
        let traj_accessor = TrajAccessorImpl { formation: &formation };
        let cont = traj.update(&traj_accessor);

        posture.0 = traj.pos();
        posture.1 = traj.angle;
        speed.0 = traj.speed;
        speed.1 = traj.vangle;
        //if let Some(wait) = traj.is_shot() {
        //    self.shot_wait = Some(wait);
        //}
        cont
    } else {
        false
    }
}

struct TrajAccessorImpl<'a> {
    formation: &'a Formation,
}
impl<'a> TrajAccessor for TrajAccessorImpl<'a> {
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }
    fn get_stage_no(&self) -> u16 { 0 }
}

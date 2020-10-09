use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::formation_table::{X_COUNT, Y_COUNT};
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::traj::Accessor as TrajAccessor;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command::TrajCommand;
use galangua_common::app::game::traj_command_table::*;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::util::math::{atan2_lut, calc_velocity, clamp, diff_angle, normalize_angle, square, ANGLE, ONE, ONE_BIT};

use crate::app::components::*;
use crate::app::resources::GameInfo;

use super::system_effect::*;
use super::system_owl::set_owl_damage;

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

pub fn set_enemy_damage<'a>(
    entity: Entity, power: u32, entities: &Entities<'a>,
    enemy_storage: &mut WriteStorage<'a, Enemy>,
    pos_storage: &mut WriteStorage<'a, Posture>,
    owl_storage: &mut WriteStorage<'a, Owl>,
    troops_storage: &mut WriteStorage<'a, Troops>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    seqanime_storage: &mut WriteStorage<'a, SequentialSpriteAnime>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
    recaptured_fighter_storage: &mut WriteStorage<'a, RecapturedFighter>,
    player_storage: &mut WriteStorage<'a, Player>,
    tractor_beam_storage: &mut WriteStorage<'a, TractorBeam>,
    attack_manager: &mut AttackManager,
    star_manager: &mut StarManager,
    game_info: &mut GameInfo,
    player_entity: Entity,
) {
    let dead = match enemy_storage.get(entity).unwrap().enemy_type {
        EnemyType::Owl => {
            let owl = owl_storage.get_mut(entity).unwrap();
            set_owl_damage(
                owl, entity, power, entities, enemy_storage, troops_storage, pos_storage,
                coll_rect_storage, drawable_storage, recaptured_fighter_storage, player_storage,
                tractor_beam_storage, attack_manager, star_manager, game_info, player_entity)
        }
        _ => {
            entities.delete(entity).unwrap();
            true
        }
    };
    if dead {
        let pos = pos_storage.get(entity).unwrap().0.clone();
        create_enemy_explosion_effect(&pos, entities, pos_storage, seqanime_storage, drawable_storage);

        game_info.decrement_alive_enemy();
    }
}

//

pub fn move_zako<'a>(
    zako: &mut Zako, entity: Entity, enemy: &mut Enemy, speed: &mut Speed,
    formation: &Formation, player_storage: &ReadStorage<'a, Player>,
    pos_storage: &mut WriteStorage<'a, Posture>,
    entities: &Entities<'a>, game_info: &mut GameInfo,
) {
    match zako.state {
        ZakoState::Appearance => {
            let cont = if let Some(traj) = &mut zako.traj.as_mut() {
                let posture = pos_storage.get_mut(entity).unwrap();
                update_traj(traj, posture, speed, formation)
            } else {
                false
            };
            if !cont {
                zako.traj = None;
                if enemy.formation_index.1 >= Y_COUNT as u8 {  // Assault
                    set_zako_assault(zako, speed, player_storage, pos_storage);
                } else {
                    zako.state = ZakoState::MoveToFormation;
                }
            }
        }
        ZakoState::Formation => {
            let mut posture = pos_storage.get_mut(entity).unwrap();
            posture.0 = formation.pos(&enemy.formation_index);

            let ang = ANGLE * ONE / 128;
            posture.1 -= clamp(posture.1, -ang, ang);
        }
        ZakoState::Attack => {
            let cont = if let Some(traj) = &mut zako.traj.as_mut() {
                let posture = pos_storage.get_mut(entity).unwrap();
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
            let posture = pos_storage.get_mut(entity).unwrap();
            let result = move_to_formation(posture, speed, &enemy.formation_index, formation);
            forward(posture, speed);
            if result {
                zako.state = ZakoState::Formation;
                enemy.is_formation = true;
            }
        }
        ZakoState::Assault(phase) => {
            let posture = pos_storage.get_mut(entity).unwrap();
            if let Some(new_phase) = update_assault(zako, posture, phase, entity, entities, game_info) {
                zako.state = ZakoState::Assault(new_phase);
            }
            forward(posture, speed);
        }
        ZakoState::Troop => {
            // Controlled by leader.
        }
    }
}

fn set_zako_assault<'a>(zako: &mut Zako, speed: &mut Speed, player_storage: &ReadStorage<'a, Player>, pos_storage: &WriteStorage<'a, Posture>) {
    /*let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
    let target_pos = [
        Some(*accessor.get_player_pos()),
        accessor.get_dual_player_pos(),
    ];
    let count = target_pos.iter().flat_map(|x| x).count();
    let target: &Vec2I = target_pos.iter()
        .flat_map(|x| x).nth(rng.gen_range(0, count)).unwrap();*/

    for (_player, posture) in (player_storage, pos_storage).join() {
        zako.target_pos = posture.0.clone();
        speed.1 = 0;
        break;
    }

    zako.state = ZakoState::Assault(0);
}

fn update_assault<'a>(zako: &mut Zako, posture: &mut Posture, phase: u32, entity: Entity, entities: &Entities<'a>, game_info: &mut GameInfo) -> Option<u32> {
    let pos = &mut posture.0;
    let angle = &mut posture.1;
    match phase {
        0 => {
            let target = &zako.target_pos;
            let diff = target - pos;

            const DLIMIT: i32 = 5 * ONE;
            let target_angle = atan2_lut(-diff.y, diff.x);
            let d = diff_angle(target_angle, *angle);
            if d < -DLIMIT {
                *angle -= DLIMIT;
            } else if d > DLIMIT {
                *angle += DLIMIT;
            } else {
                *angle += d;
                return Some(1);
            }
        }
        1 | _ => {
            if pos.y >= (HEIGHT + 8) * ONE {
                entities.delete(entity).unwrap();
                game_info.decrement_alive_enemy();
            }
        }
    }
    None
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

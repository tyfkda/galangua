use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

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

pub fn set_enemy_damage(
    enemy_type: EnemyType, entity: Entity, power: u32,
    player_entity: Entity,
    star_manager: &mut StarManager,
    attack_manager: &mut AttackManager,
    game_info: &mut GameInfo,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    let dead = match enemy_type {
        EnemyType::Owl => {
            let (mut subworld1, mut subworld2) = world.split::<&mut Owl>();
            let owl = <&mut Owl>::query().get_mut(&mut subworld1, entity).unwrap();
            set_owl_damage(owl, entity, power, player_entity, attack_manager, star_manager, game_info, &mut subworld2, commands)
        }
        _ => {
            commands.remove(entity);
            true
        }
    };
    if dead {
        let pos = <&Posture>::query().get(world, entity).unwrap().0;
        create_enemy_explosion_effect(&pos, commands);

        game_info.decrement_alive_enemy();
    }
}

//

pub fn do_move_zako(
    zako: &mut Zako, entity: Entity,
    enemy: &mut Enemy, speed: &mut Speed,
    formation: &Formation, game_info: &mut GameInfo,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) {
    match zako.state {
        ZakoState::Appearance => {
            let cont = if let Some(traj) = &mut zako.traj.as_mut() {
                let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
                update_traj(traj, posture, speed, formation)
            } else {
                false
            };
            if !cont {
                zako.traj = None;
                if enemy.formation_index.1 >= Y_COUNT as u8 {  // Assault
                    set_zako_assault(zako, speed, world);
                } else {
                    zako.state = ZakoState::MoveToFormation;
                }
            }
        }
        ZakoState::Formation => {
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            posture.0 = formation.pos(&enemy.formation_index);

            let ang = ANGLE * ONE / 128;
            posture.1 -= clamp(posture.1, -ang, ang);
        }
        ZakoState::Attack(t) => {
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            match t {
                ZakoAttackType::BeeAttack => {
                    update_bee_attack(zako, enemy, posture, speed, formation, game_info);
                }
                ZakoAttackType::Traj => {
                    update_attack_traj(zako, enemy, posture, speed, formation, game_info, entity, commands);
                }
            }
        }
        ZakoState::MoveToFormation => {
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            let result = move_to_formation(posture, speed, &enemy.formation_index, formation);
            forward(posture, speed);
            if result {
                zako.state = ZakoState::Formation;
                enemy.is_formation = true;
            }
        }
        ZakoState::Assault(phase) => {
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            if let Some(new_phase) = update_assault(zako, posture, phase, entity, game_info, commands) {
                zako.state = ZakoState::Assault(new_phase);
            }
            forward(posture, speed);
        }
        ZakoState::Troop => {
            // Controlled by leader.
        }
    }
}

fn set_zako_assault(zako: &mut Zako, speed: &mut Speed, world: &SubWorld) {
    /*let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
    let target_pos = [
        Some(*accessor.get_player_pos()),
        accessor.get_dual_player_pos(),
    ];
    let count = target_pos.iter().flat_map(|x| x).count();
    let target: &Vec2I = target_pos.iter()
        .flat_map(|x| x).nth(rng.gen_range(0, count)).unwrap();*/

    for (_player, posture) in <(&Player, &Posture)>::query().iter(world) {
        zako.target_pos = posture.0.clone();
        speed.1 = 0;
        break;
    }

    zako.state = ZakoState::Assault(0);
}

fn update_assault(zako: &mut Zako, posture: &mut Posture, phase: u32, entity: Entity, game_info: &mut GameInfo, commands: &mut CommandBuffer) -> Option<u32> {
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
                commands.remove(entity);
                game_info.decrement_alive_enemy();
            }
        }
    }
    None
}

pub fn zako_start_attack(zako: &mut Zako, enemy: &mut Enemy, posture: &Posture) {
    let flip_x = enemy.formation_index.0 >= (X_COUNT as u8) / 2;
    let (table, state): (&[TrajCommand], ZakoState) = match enemy.enemy_type {
        EnemyType::Bee => (&BEE_ATTACK_TABLE, ZakoState::Attack(ZakoAttackType::BeeAttack)),
        EnemyType::Butterfly => (&BUTTERFLY_ATTACK_TABLE, ZakoState::Attack(ZakoAttackType::Traj)),
        EnemyType::Owl => (&OWL_ATTACK_TABLE, ZakoState::Attack(ZakoAttackType::Traj)),
        EnemyType::CapturedFighter => (&OWL_ATTACK_TABLE, ZakoState::Attack(ZakoAttackType::Traj)),
    };
    let mut traj = Traj::new(table, &ZERO_VEC, flip_x, enemy.formation_index.clone());
    traj.set_pos(&posture.0);
    zako.traj = Some(traj);
    zako.state = state;
    enemy.is_formation = false;
}

fn update_bee_attack(zako: &mut Zako, enemy: &Enemy, posture: &mut Posture, speed: &mut Speed, formation: &Formation, game_info: &GameInfo) {
    let cont = if let Some(traj) = &mut zako.traj.as_mut() {
        update_traj(traj, posture, speed, formation)
    } else {
        false
    };
    if !cont {
        if game_info.is_rush() {
            let flip_x = enemy.formation_index.0 >= 5;
            let mut traj = Traj::new(&BEE_ATTACK_RUSH_CONT_TABLE, &ZERO_VEC, flip_x,
                                     enemy.formation_index);
            traj.set_pos(&posture.0);

            zako.traj = Some(traj);
            zako.state = ZakoState::Attack(ZakoAttackType::Traj);
        } else {
            zako.traj = None;
            zako.state = ZakoState::MoveToFormation;
        }
    }
}

fn update_attack_traj(zako: &mut Zako, enemy: &Enemy, posture: &mut Posture, speed: &mut Speed, formation: &Formation, game_info: &mut GameInfo, entity: Entity, commands: &mut CommandBuffer) {
    let cont = if let Some(traj) = &mut zako.traj.as_mut() {
        update_traj(traj, posture, speed, formation)
    } else {
        false
    };
    if !cont {
        zako.traj = None;
        if enemy.enemy_type == EnemyType::CapturedFighter {
            commands.remove(entity);
            game_info.decrement_alive_enemy();
        } else if game_info.is_rush() {
            // Rush mode: Continue attacking
            let table = VTABLE[enemy.enemy_type as usize].rush_traj_table;
            rush_attack(zako, table, posture, &enemy.formation_index);
            //accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
        } else {
            zako.state = ZakoState::MoveToFormation;
        }
    }
}

fn rush_attack(zako: &mut Zako, table: &'static [TrajCommand], posture: &Posture, fi: &FormationIndex) {
    let flip_x = fi.0 >= 5;
    let mut traj = Traj::new(table, &ZERO_VEC, flip_x, *fi);
    traj.set_pos(&posture.0);

    //self.count = 0;
    //self.attack_frame_count = 0;
    zako.traj = Some(traj);
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

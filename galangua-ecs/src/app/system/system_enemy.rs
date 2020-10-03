use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::formation_table::X_COUNT;
use galangua_common::app::game::tractor_beam_table::*;
use galangua_common::app::game::traj::Accessor as TrajAccessor;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command::TrajCommand;
use galangua_common::app::game::traj_command_table::*;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::util::math::{atan2_lut, calc_velocity, clamp, diff_angle, normalize_angle, square, ANGLE, ONE, ONE_BIT};

use crate::app::components::*;
use crate::app::resources::*;

fn forward(posture: &mut Posture, speed: &Speed) {
    posture.0 += &calc_velocity(posture.1 + speed.1 / 2, speed.0);
    posture.1 += speed.1;
}

pub fn move_zako(zako: &mut Zako, enemy: &mut Enemy, posture: &mut Posture, speed: &mut Speed, formation: &Formation) {
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
                forward(posture, speed);
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
                enemy.is_formation = true;
            }
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

pub fn create_owl(traj: Traj) -> Owl {
    Owl {
        state: OwlState::Appearance,
        traj: Some(traj),
        capturing_state: OwlCapturingState::None,
        target_pos: ZERO_VEC,
        tractor_beam: None,
    }
}

pub fn move_owl<'a>(
    owl: &mut Owl, enemy: &mut Enemy,
    posture: &mut Posture, speed: &mut Speed,
    formation: &Formation,
    entities: &Entities<'a>,
    tractor_beam_storage: &mut WriteStorage<'a, TractorBeam>,
    game_info: &mut GameInfo,
) {
    match owl.state {
        OwlState::Appearance => {
            let cont = if let Some(traj) = &mut owl.traj.as_mut() {
                update_traj(traj, posture, speed, formation)
            } else {
                false
            };
            if !cont {
                owl.traj = None;
                owl.state = OwlState::MoveToFormation;
            }
        }
        OwlState::Formation => {
            posture.0 = formation.pos(&enemy.formation_index);

            let ang = ANGLE * ONE / 128;
            posture.1 -= clamp(posture.1, -ang, ang);
        }
        OwlState::TrajAttack => {
            let cont = if let Some(traj) = &mut owl.traj.as_mut() {
                update_traj(traj, posture, speed, formation)
            } else {
                false
            };
            if !cont {
                owl.traj = None;
                owl.state = OwlState::MoveToFormation;
            }
        }
        OwlState::CaptureAttack(phase) => {
            run_capture_attack(phase, owl, enemy, posture, speed, &formation, entities, tractor_beam_storage, game_info);
            forward(posture, speed);
        }
        OwlState::MoveToFormation => {
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
                *pos += &calc_velocity(*angle + *vangle / 2, *spd);
                true
            } else {
                *pos = target;
                *spd = 0;
                *angle = normalize_angle(*angle);
                *vangle = 0;
                false
            };
            if !cont {
                owl.state = OwlState::Formation;
                enemy.is_formation = true;
            }
        }
    }
}

pub fn owl_start_attack(owl: &mut Owl, enemy: &mut Enemy, posture: &mut Posture, capture_attack: bool, speed: &mut Speed, player_pos: &Vec2I) {
    let flip_x = enemy.formation_index.0 >= (X_COUNT as u8) / 2;
    if !capture_attack {
        let table: &[TrajCommand] = &OWL_ATTACK_TABLE;
        let mut traj = Traj::new(table, &ZERO_VEC, flip_x, enemy.formation_index.clone());
        traj.set_pos(&posture.0);
        owl.traj = Some(traj);
        owl.state = OwlState::TrajAttack;
    } else {
        owl.capturing_state = OwlCapturingState::Attacking;

        const DLIMIT: i32 = 4 * ONE;
        speed.0 = 3 * ONE / 2;
        posture.1 = 0;
        if !flip_x {
            speed.1 = -DLIMIT;
        } else {
            speed.1 = DLIMIT;
        }

        owl.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

        owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::Capture);
    }
    enemy.is_formation = false;
}

fn run_capture_attack<'a>(
    phase: OwlCaptureAttackPhase,
    owl: &mut Owl, enemy: &mut Enemy,
    posture: &mut Posture, speed: &mut Speed,
    formation: &Formation,
    entities: &Entities<'a>,
    tractor_beam_storage: &mut WriteStorage<'a, TractorBeam>,
    game_info: &mut GameInfo,
) {
    let target_pos = &owl.target_pos;
    let pos = &mut posture.0;
    let angle = &mut posture.1;
    let spd = &mut speed.0;
    let vangle = &mut speed.1;
    match phase {
        OwlCaptureAttackPhase::Capture => {
            const DLIMIT: i32 = 4 * ONE;
            let dpos = target_pos - pos;
            let target_angle = atan2_lut(-dpos.y, dpos.x);
            let ang_limit = ANGLE * ONE / 2 - ANGLE * ONE * 30 / 360;
            let target_angle = if target_angle >= 0 {
                std::cmp::max(target_angle, ang_limit)
            } else {
                std::cmp::min(target_angle, -ang_limit)
            };
            let mut d = diff_angle(target_angle, *angle);
            if *vangle > 0 && d < 0 {
                d += ANGLE * ONE;
            } else if *vangle < 0 && d > 0 {
                d -= ANGLE * ONE;
            }
            if d >= -DLIMIT && d < DLIMIT {
                *angle = target_angle;
                *vangle = 0;
            }

            if pos.y >= target_pos.y {
                pos.y = target_pos.y;
                *spd = 0;
                *angle = ANGLE / 2 * ONE;
                *vangle = 0;

                let tractor_beam = entities.build_entity()
                    .with(create_tractor_beam(&(&*pos + &Vec2I::new(0, 8 * ONE))), tractor_beam_storage)
                    .build();
                owl.tractor_beam = Some(tractor_beam);

                //accessor.push_event(EventType::PlaySe(CH_JINGLE, SE_TRACTOR_BEAM1));

                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureBeam);
                //self.base.count = 0;
            }
        }
        OwlCaptureAttackPhase::CaptureBeam => {
            let tractor_beam = tractor_beam_storage.get(owl.tractor_beam.unwrap()).unwrap();
            if tractor_beam_closed(tractor_beam) {
                owl.tractor_beam = None;
                *spd = 5 * ONE / 2;
                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::NoCaptureGoOut);
            /*} else if accessor.can_player_capture() &&
                        tractor_beam.can_capture(accessor.get_player_pos())
            {
                accessor.push_event(EventType::CapturePlayer(&self.info.pos + &Vec2I::new(0, 16 * ONE)));
                accessor.push_event(EventType::PlaySe(CH_JINGLE, SE_TRACTOR_BEAM2));
                tractor_beam.start_capture();
                self.capturing_state = CapturingState::BeamTracting;
                self.set_state(OwlState::Attack(OwlAttackPhase::CaptureStart));
                self.base.count = 0;*/
            }
        }
        OwlCaptureAttackPhase::NoCaptureGoOut => {
            if pos.y >= (HEIGHT + 8) * ONE {
                let target_pos = formation.pos(&enemy.formation_index);
                let offset = Vec2I::new(target_pos.x - pos.x, (-32 - (HEIGHT + 8)) * ONE);
                *pos += &offset;

                /*if accessor.is_rush() {
                    self.rush_attack();
                    accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
                } else*/ {
                    owl.state = OwlState::MoveToFormation;
                    owl.capturing_state = OwlCapturingState::Failed;
                    //accessor.push_event(EventType::EndCaptureAttack);
                    game_info.end_capture_attack();
                }
            }
        }
        OwlCaptureAttackPhase::CaptureStart => {

        }
        OwlCaptureAttackPhase::CaptureCloseBeam => {

        }
        OwlCaptureAttackPhase::CaptureDoneWait => {

        }
        OwlCaptureAttackPhase::CaptureDoneBack => {

        }
        OwlCaptureAttackPhase::CaptureDonePushUp => {

        }
    }
}

fn create_tractor_beam(pos: &Vec2I) -> TractorBeam {
    TractorBeam {
        pos: pos + &Vec2I::new(-24 * ONE, 0),
        state: TractorBeamState::Opening,
        count: 0,
        color_count: 0,
        size_count: 0,
        beam_sprites: [None; TRACTOR_BEAM_SPRITE_COUNT],
    }
}

pub fn move_tractor_beam<'a>(tractor_beam: &mut TractorBeam, entities: &Entities<'a>, pos_storage: &mut WriteStorage<'a, Posture>, sprite_storage: &mut WriteStorage<'a, SpriteDrawable>) {
    use TractorBeamState::*;

    tractor_beam.color_count += 1;

    match tractor_beam.state {
        Opening => {
            let pn = tractor_beam.size_count / ONE;
            tractor_beam.size_count += ONE / 3;
            let an = tractor_beam.size_count / ONE;
            if an != pn {
                let i = pn as usize;
                let sprite = SpriteDrawable { sprite_name: TRACTOR_BEAM_SPRITE_NAMES[i], offset: ZERO_VEC };
                let posture = Posture(&tractor_beam.pos + &Vec2I::new(0, TRACTOR_BEAM_Y_OFFSET_TABLE[i] * ONE), 0);
                let entity = entities.build_entity()
                    .with(sprite, sprite_storage)
                    .with(posture, pos_storage)
                    .build();
                tractor_beam.beam_sprites[i] = Some(entity);

                if an as usize >= TRACTOR_BEAM_SPRITE_NAMES.len() {
                    tractor_beam.size_count = an * ONE;
                    tractor_beam.state = Full;
                    tractor_beam.count = 0;
                }
            }
        }
        Full => {
            tractor_beam.count += 1;
            if tractor_beam.count >= 3 * 60 {
                tractor_beam.state = Closing;
            }
        }
        Closing => {
            let pn = (tractor_beam.size_count + ONE - 1) / ONE;
            if tractor_beam.size_count > ONE / 3 {
                tractor_beam.size_count -= ONE / 3;
            } else {
                tractor_beam.size_count = 0;
            }
            let an = (tractor_beam.size_count + ONE - 1) / ONE;
            if an != pn {
                let i = an as usize;
                if let Some(entity) = tractor_beam.beam_sprites[i].take() {
                    entities.delete(entity).unwrap();
                }

                if an == 0 {
                    tractor_beam.state = Closed;
                }
            }
        }
        Closed => {}
        Capturing => {}
    }
}

fn tractor_beam_closed(tractor_beam: &TractorBeam) -> bool {
    tractor_beam.state == TractorBeamState::Closed
}

fn update_traj(traj: &mut Traj, posture: &mut Posture, vel: &mut Speed, formation: &Formation) -> bool {
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

struct TrajAccessorImpl<'a> {
    formation: &'a Formation,
}
impl<'a> TrajAccessor for TrajAccessorImpl<'a> {
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }
    fn get_stage_no(&self) -> u16 { 0 }
}

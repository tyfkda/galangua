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

use super::system_player::{move_capturing_player, set_player_captured, start_player_capturing};

fn forward(posture: &mut Posture, speed: &Speed) {
    posture.0 += &calc_velocity(posture.1 + speed.1 / 2, speed.0);
    posture.1 += speed.1;
}

fn move_to_formation(posture: &mut Posture, speed: &mut Speed, fi: &FormationIndex, formation: &Formation) -> bool {
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

pub fn create_owl(traj: Traj) -> Owl {
    Owl {
        state: OwlState::Appearance,
        traj: Some(traj),
        capturing_state: OwlCapturingState::None,
        target_pos: ZERO_VEC,
        count: 0,
    }
}

pub fn move_owl<'a>(
    owl: &mut Owl, entity: Entity, enemy: &mut Enemy,
    posture: &mut Posture, speed: &mut Speed,
    formation: &Formation,
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
            run_capture_attack(owl, entity, phase, enemy, posture, speed, &formation, tractor_beam_storage, game_info);
            forward(posture, speed);
        }
        OwlState::MoveToFormation => {
            let result = move_to_formation(posture, speed, &enemy.formation_index, formation);
            forward(posture, speed);
            if result {
                owl.capturing_state = OwlCapturingState::None;
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
    owl: &mut Owl,
    entity: Entity,
    phase: OwlCaptureAttackPhase,
    enemy: &mut Enemy,
    posture: &mut Posture, speed: &mut Speed,
    formation: &Formation,
    tractor_beam_storage: &mut WriteStorage<'a, TractorBeam>,
    game_info: &mut GameInfo,
) {
    match phase {
        OwlCaptureAttackPhase::Capture => {
            let target_pos = &owl.target_pos;
            let pos = &mut posture.0;
            let angle = &mut posture.1;
            let spd = &mut speed.0;
            let vangle = &mut speed.1;

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

                tractor_beam_storage.insert(entity, create_tractor_beam(&(&*pos + &Vec2I::new(0, 8 * ONE)))).unwrap();

                //accessor.push_event(EventType::PlaySe(CH_JINGLE, SE_TRACTOR_BEAM1));

                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureBeam);
                //self.base.count = 0;
            }
        }
        OwlCaptureAttackPhase::CaptureBeam => {
            let spd = &mut speed.0;

            let tractor_beam = tractor_beam_storage.get_mut(entity).unwrap();
            if tractor_beam_closed(tractor_beam) {
                tractor_beam_storage.remove(entity);
                *spd = 5 * ONE / 2;
                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::NoCaptureGoOut);
            } else if tractor_beam_start_capturing(tractor_beam) {
                //accessor.push_event(EventType::CapturePlayer(&self.info.pos + &Vec2I::new(0, 16 * ONE)));
                game_info.capture_player();
                //accessor.push_event(EventType::PlaySe(CH_JINGLE, SE_TRACTOR_BEAM2));
                start_capturing(tractor_beam);

                owl.capturing_state = OwlCapturingState::BeamTracting;
                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::Capturing);
                //self.base.count = 0;*/
            }
        }
        OwlCaptureAttackPhase::NoCaptureGoOut => {
            let pos = &mut posture.0;

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
                    game_info.end_capture_attack();
                }
            }
        }
        OwlCaptureAttackPhase::Capturing => {
            // Handled in TractorBeam.
        }
        OwlCaptureAttackPhase::CaptureDoneWait => {
            let spd = &mut speed.0;

            owl.count += 1;
            if owl.count >= 120 {
                *spd = 5 * ONE / 2;
                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureDoneBack);
            }
        }
        OwlCaptureAttackPhase::CaptureDoneBack => {
            if move_to_formation(posture, speed, &enemy.formation_index, formation) {
                let angle = &mut posture.1;
                let spd = &mut speed.0;

                *spd = 0;
                *angle = normalize_angle(*angle);
                owl.capturing_state = OwlCapturingState::None;
                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureDonePushUp);
            }
        }
        OwlCaptureAttackPhase::CaptureDonePushUp => {
            let angle = &mut posture.1;

            let ang = ANGLE * ONE / 128;
            *angle -= clamp(*angle, -ang, ang);

            // Cannot touch troops' posture here.

            /*
            let mut done = false;
            let fi = FormationIndex(enemy.formation_index.0, enemy.formation_index.1 - 1);
            let captured_fighter = accessor.get_enemy_at_mut(&fi).unwrap();
            let mut pos = *captured_fighter.pos();
            pos.y -= 1 * ONE;
            let topy = self.info.pos.y - 16 * ONE;
            if pos.y <= topy {
                pos.y = topy;
                done = true;
            }
            captured_fighter.set_pos(&pos);

            if done {
                //accessor.push_event(EventType::CaptureSequenceEnded);
                //self.release_troops(accessor);
                //self.set_to_formation();
                owl.state = OwlState::Formation;
                enemy.is_formation = true;
            }
            */
        }
    }
}

fn set_owl_capturing_player_completed(owl: &mut Owl) {
    owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureDoneWait);
    owl.count = 0;
}

fn create_tractor_beam(pos: &Vec2I) -> TractorBeam {
    TractorBeam {
        pos: pos.clone(),
        state: TractorBeamState::Opening,
        count: 0,
        color_count: 0,
        size_count: 0,
        beam_sprites: [None; TRACTOR_BEAM_SPRITE_COUNT],
        capturing_player: None,
    }
}

pub fn move_tractor_beam<'a>(
    tractor_beam: &mut TractorBeam,
    game_info: &mut GameInfo,
    entities: &Entities<'a>, _entity: Entity,
    owl: &mut Owl,
    pos_storage: &mut WriteStorage<'a, Posture>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
    player_storage: &mut WriteStorage<'a, Player>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    enemy_storage: &mut WriteStorage<'a, Enemy>,
    zako_storage: &mut WriteStorage<'a, Zako>,
    speed_storage: &mut WriteStorage<'a, Speed>,
    troops_storage: &mut WriteStorage<'a, Troops>,
) {
    use TractorBeamState::*;

    tractor_beam.color_count += 1;

    match tractor_beam.state {
        Opening => {
            let pn = tractor_beam.size_count / ONE;
            tractor_beam.size_count += ONE / 3;
            let an = tractor_beam.size_count / ONE;
            if an != pn {
                let i = pn as usize;
                let sprite = SpriteDrawable { sprite_name: TRACTOR_BEAM_SPRITE_NAMES[i], offset: Vec2I::new(-24, 0) };
                let posture = Posture(&tractor_beam.pos + &Vec2I::new(0, TRACTOR_BEAM_Y_OFFSET_TABLE[i] * ONE), 0);
                let entity = entities.build_entity()
                    .with(sprite, drawable_storage)
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
            if let Some(entity) = can_tractor_beam_capture_player(game_info, &tractor_beam.pos, player_storage, pos_storage, entities) {
                let player = player_storage.get_mut(entity).unwrap();
                start_player_capturing(player, entity, coll_rect_storage);
                game_info.start_capturing();
                tractor_beam.capturing_player = Some(entity);
                tractor_beam.state = TractorBeamState::Capturing;
                tractor_beam.count = 0;
                return;
            }

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
                    if tractor_beam.capturing_player.is_some() {
                        on_capturing_player_completed(owl);
                    }
                    tractor_beam.state = Closed;
                }
            }
        }
        Closed => {}
        Capturing => {
            let entity = tractor_beam.capturing_player.unwrap();
            let player = player_storage.get_mut(entity).unwrap();
            let posture = pos_storage.get_mut(entity).unwrap();
            if move_capturing_player(player, posture, &(&tractor_beam.pos + &Vec2I::new(0, 8 * ONE))) {
                on_player_captured(
                    &tractor_beam.pos, entity, entity, entities,
                    enemy_storage, zako_storage, pos_storage, speed_storage,
                    coll_rect_storage, drawable_storage, troops_storage);
                tractor_beam.state = TractorBeamState::Closing;
            }
        }
    }
}

fn can_tractor_beam_capture_player<'a>(game_info: &GameInfo, beam_pos: &Vec2I, player_storage: &mut WriteStorage<'a, Player>, pos_storage: &WriteStorage<'a, Posture>, entities: &Entities<'a>) -> Option<Entity> {
    const RANGE: i32 = 24 * ONE;
    for (_player, player_pos, entity) in (player_storage, pos_storage, entities).join() {
        if game_info.can_capture() &&
            (player_pos.0.x - beam_pos.x).abs() <= RANGE
        {
            return Some(entity)
        }
    }
    None
}

fn on_player_captured<'a>(
    pos: &Vec2I,
    owner: Entity,
    player: Entity,
    entities: &Entities<'a>,
    enemy_storage: &mut WriteStorage<'a, Enemy>,
    zako_storage: &mut WriteStorage<'a, Zako>,
    pos_storage: &mut WriteStorage<'a, Posture>,
    speed_storage: &mut WriteStorage<'a, Speed>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
    troops_storage: &mut WriteStorage<'a, Troops>,
) {
    set_player_captured(player, drawable_storage);

    let enemy = enemy_storage.get(owner).unwrap();
    let fi = FormationIndex(enemy.formation_index.0, enemy.formation_index.1 - 1);
    let captured = entities.build_entity()
        .with(Enemy { enemy_type: EnemyType::CapturedFighter, formation_index: fi, is_formation: false }, enemy_storage)
        .with(Posture(pos + &Vec2I::new(0, 8 * ONE), 0), pos_storage)
        .with(Speed(0, 0), speed_storage)
        .with(CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) }, coll_rect_storage)
        .with(SpriteDrawable {sprite_name: "rustacean_captured", offset: Vec2I::new(-8, -8)}, drawable_storage)
        .with(Zako { state: ZakoState::Troop, traj: None }, zako_storage)
        .build();

    let mut troops = Troops {members: Default::default()};
    add_captured_player_to_troops(&mut troops, captured, &Vec2I::new(0, 16 * ONE));
    troops_storage.insert(owner, troops).unwrap();
}

fn on_capturing_player_completed<'a>(owl: &mut Owl) {
    set_owl_capturing_player_completed(owl);
}

fn tractor_beam_closed(tractor_beam: &TractorBeam) -> bool {
    tractor_beam.state == TractorBeamState::Closed
}

fn tractor_beam_start_capturing(tractor_beam: &TractorBeam) -> bool {
    tractor_beam.state == TractorBeamState::Capturing
}

fn start_capturing(tractor_beam: &mut TractorBeam) {
    tractor_beam.state = TractorBeamState::Capturing;
}

// Troops

pub fn update_troops<'a>(
    troops: &mut Troops, owner: &Entity, owl_storage: &ReadStorage<'a, Owl>,
    pos_storage: &mut WriteStorage<'a, Posture>,
) {
    assert!(owl_storage.get(*owner).is_some());
    let &Posture(pos, angle) = pos_storage.get(*owner).unwrap();
    for member in troops.members.iter().flat_map(|x| x) {
        if let Some(member_pos) = pos_storage.get_mut(member.0) {
            member_pos.0 = &pos + &member.1;
            member_pos.1 = angle;
        }
    }
}

fn add_captured_player_to_troops(troops: &mut Troops, captured: Entity, offset: &Vec2I) {
    // On capture attack, must be no troops exists.
    assert!(troops.members[0].is_none());
    troops.members[0] = Some((captured, offset.clone()));
}

fn release_owl_troops<'a>(troops: &mut Troops, enemy_storage: &mut WriteStorage<'a, Enemy>, zako_storage: &mut WriteStorage<'a, Zako>) {
    for member_opt in troops.members.iter_mut().filter(|x| x.is_some()) {
        let member = member_opt.unwrap();
        if let Some(enemy) = enemy_storage.get_mut(member.0) {
            enemy.is_formation = true;
        }
        if let Some(zako) = zako_storage.get_mut(member.0) {
            zako.state = ZakoState::Formation;
        }
        *member_opt = None;
    }
}

//

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
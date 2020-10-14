use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::formation_table::X_COUNT;
use galangua_common::app::game::tractor_beam_table::*;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command::TrajCommand;
use galangua_common::app::game::traj_command_table::*;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::util::math::{atan2_lut, clamp, diff_angle, normalize_angle, ANGLE, ONE};

use crate::app::components::*;
use crate::app::resources::*;

use super::system_enemy::{forward, move_to_formation, set_zako_to_troop, update_traj};
use super::system_player::{move_capturing_player, set_player_captured, start_player_capturing};

// Owl

pub fn create_owl(traj: Traj) -> Owl {
    Owl {
        state: OwlState::Appearance,
        traj: Some(traj),
        capturing_state: OwlCapturingState::None,
        target_pos: ZERO_VEC,
        //tractor_beam: None,
        count: 0,
    }
}

pub fn do_move_owl(
    owl: &mut Owl, entity: Entity,
    posture: &mut Posture, speed: &mut Speed,
    formation: &Formation,
    game_info: &mut GameInfo,
    world: &mut SubWorld, commands: &mut CommandBuffer,
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
            let fi = &<&Enemy>::query().get(world, entity).unwrap().formation_index;
            posture.0 = formation.pos(fi);

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
            let (mut subworld1, mut subworld2) = world.split::<&mut TractorBeam>();
            let tractor_beam_opt = <&mut TractorBeam>::query().get_mut(&mut subworld1, entity).ok();
            run_capture_attack(owl, entity, phase, posture, speed, tractor_beam_opt, formation, game_info, &mut subworld2, commands);
            forward(posture, speed);
        }
        OwlState::MoveToFormation => {
            let result = {
                let fi = &<&Enemy>::query().get(world, entity).unwrap().formation_index;
                let result = move_to_formation(posture, speed, fi, formation);
                forward(posture, speed);
                result
            };
            if result {
                {
                    let (mut subworld1, mut subworld2) = world.split::<&mut Troops>();
                    if let Ok(troops) = <&mut Troops>::query().get_mut(&mut subworld1, entity) {
                        release_troops(troops, &mut subworld2);
                        commands.remove_component::<Troops>(entity);
                    }
                }

                owl.capturing_state = OwlCapturingState::None;
                owl.state = OwlState::Formation;

                let enemy = <&mut Enemy>::query().get_mut(world, entity).unwrap();
                enemy.is_formation = true;
            }
        }
    }
}

pub fn owl_start_attack(
    owl: &mut Owl, capture_attack: bool, speed: &mut Speed, player_pos: &Vec2I,
    entity: Entity,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) {
    let fi = <&Enemy>::query().get(world, entity).unwrap().formation_index.clone();
    let flip_x = fi.0 >= (X_COUNT as u8) / 2;
    if !capture_attack {
        let pos = <&Posture>::query().get(world, entity).unwrap().0.clone();
        choose_troops(entity, &fi, &pos, world, commands);

        let posture = <&Posture>::query().get(world, entity).unwrap();
        let table: &[TrajCommand] = &OWL_ATTACK_TABLE;
        let mut traj = Traj::new(table, &ZERO_VEC, flip_x, fi);
        traj.set_pos(&posture.0);
        owl.traj = Some(traj);
        owl.state = OwlState::TrajAttack;
    } else {
        owl.capturing_state = OwlCapturingState::Attacking;

        const DLIMIT: i32 = 4 * ONE;
        speed.0 = 3 * ONE / 2;
        let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
        posture.1 = 0;
        if fi.0 < (X_COUNT as u8) / 2 {
            speed.1 = -DLIMIT;
        } else {
            speed.1 = DLIMIT;
        }

        owl.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

        owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::Capture);
    }

    let enemy = <&mut Enemy>::query().get_mut(world, entity).unwrap();
    enemy.is_formation = false;
}

fn run_capture_attack(
    owl: &mut Owl, entity: Entity,
    phase: OwlCaptureAttackPhase,
    posture: &mut Posture, speed: &mut Speed,
    tractor_beam_opt: Option<&mut TractorBeam>,
    formation: &Formation,
    game_info: &mut GameInfo,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
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

                commands.add_component(
                    entity,
                    create_tractor_beam(&(&*pos + &Vec2I::new(0, 8 * ONE))));

                //accessor.push_event(EventType::PlaySe(CH_JINGLE, SE_TRACTOR_BEAM1));

                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureBeam);
                //self.base.count = 0;
            }
        }
        OwlCaptureAttackPhase::CaptureBeam => {
            let spd = &mut speed.0;

            let tractor_beam = tractor_beam_opt.unwrap();
            if tractor_beam_closed(tractor_beam) {
                //owl.tractor_beam = None;
                commands.remove_component::<TractorBeam>(entity);
                *spd = 5 * ONE / 2;
                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::NoCaptureGoOut);
            } else if tractor_beam_start_capturing(tractor_beam) {
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
                let enemy = <&mut Enemy>::query().get_mut(world, entity).unwrap();
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
            let enemy = <&mut Enemy>::query().get_mut(world, entity).unwrap();
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
            let (mut subworld1, mut subworld2) = world.split::<&mut Troops>();
            let troops = <&mut Troops>::query().get_mut(&mut subworld1, entity).unwrap();
            let done = {
                let angle = &mut posture.1;

                let ang = ANGLE * ONE / 128;
                *angle -= clamp(*angle, -ang, ang);

                // Cannot touch troops' posture here.

                let captured_fighter = troops.members[0].as_mut().unwrap();
                //let fi = FormationIndex(enemy.formation_index.0, enemy.formation_index.1 - 1);
                //let captured_fighter = accessor.get_enemy_at_mut(&fi).unwrap();
                let pos = &mut captured_fighter.1;
                pos.y -= 1 * ONE;
                let topy = -16 * ONE;
                if pos.y <= topy {
                    pos.y = topy;
                    true
                } else {
                    false
                }
            };

            if done {
                //accessor.push_event(EventType::CaptureSequenceEnded);
                release_troops(troops, &mut subworld2);

                //self.set_to_formation();
                owl.state = OwlState::Formation;

                let enemy = <&mut Enemy>::query().get_mut(world, entity).unwrap();
                enemy.is_formation = true;
            }
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

pub fn do_move_tractor_beam(
    tractor_beam: &mut TractorBeam, entity: Entity,
    owl: &mut Owl, enemy: &Enemy,
    game_info: &mut GameInfo,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
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
                let entity = commands.push((
                    SpriteDrawable { sprite_name: TRACTOR_BEAM_SPRITE_NAMES[i], offset: Vec2I::new(-24, 0) },
                    Posture(&tractor_beam.pos + &Vec2I::new(0, TRACTOR_BEAM_Y_OFFSET_TABLE[i] * ONE), 0),
                ));
                tractor_beam.beam_sprites[i] = Some(entity);

                if an as usize >= TRACTOR_BEAM_SPRITE_NAMES.len() {
                    tractor_beam.size_count = an * ONE;
                    tractor_beam.state = Full;
                    tractor_beam.count = 0;
                }
            }
        }
        Full => {
            if let Some(player_entity) = can_tractor_beam_capture_player(game_info, &tractor_beam.pos, world) {
                let player = <&mut Player>::query().get_mut(world, player_entity).unwrap();
                start_player_capturing(player, player_entity, commands);
                game_info.start_capturing();
                tractor_beam.capturing_player = Some(player_entity);
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
                    commands.remove(entity);
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
            let player_entity = tractor_beam.capturing_player.unwrap();
            let (player, posture) = <(&mut Player, &mut Posture)>::query().iter_mut(world).find(|_| true).unwrap();
            if move_capturing_player(player, posture, &(&tractor_beam.pos + &Vec2I::new(0, 8 * ONE))) {
                on_player_captured(
                    enemy, &tractor_beam.pos, entity, player_entity, commands);
                tractor_beam.state = TractorBeamState::Closing;
            }
        }
    }
}

fn can_tractor_beam_capture_player(
    game_info: &GameInfo, beam_pos: &Vec2I,
    world: &SubWorld,
) -> Option<Entity> {
    const RANGE: i32 = 24 * ONE;
    for (_player, player_pos, entity) in <(&Player, &Posture, Entity)>::query().iter(world) {
        if game_info.can_capture() &&
            (player_pos.0.x - beam_pos.x).abs() <= RANGE
        {
            return Some(*entity)
        }
    }
    None
}

fn on_player_captured(
    enemy: &Enemy,
    pos: &Vec2I,
    owner: Entity,
    player: Entity,
    commands: &mut CommandBuffer,
) {
    set_player_captured(player, commands);

    let fi = FormationIndex(enemy.formation_index.0, enemy.formation_index.1 - 1);
    let captured = commands.push((
        Enemy { enemy_type: EnemyType::CapturedFighter, formation_index: fi, is_formation: false },
        Zako { state: ZakoState::Troop, traj: None },
        Posture(pos + &Vec2I::new(0, 8 * ONE), 0),
        Speed(0, 0),
        CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) },
        SpriteDrawable {sprite_name: "rustacean_captured", offset: Vec2I::new(-8, -8)},
    ));

    let mut troops = Troops {members: Default::default()};
    add_captured_player_to_troops(&mut troops, captured, &Vec2I::new(0, 16 * ONE));
    commands.add_component(owner, troops);
}

fn on_capturing_player_completed(owl: &mut Owl) {
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

fn choose_troops(
    leader_entity: Entity,
    leader_fi: &FormationIndex,
    leader_pos: &Vec2I,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    let indices = [
        FormationIndex(leader_fi.0 - 1, leader_fi.1 + 1),
        FormationIndex(leader_fi.0 + 1, leader_fi.1 + 1),
        FormationIndex(leader_fi.0, leader_fi.1 - 1),
    ];
    let mut troops = Troops { members: Default::default() };
    let mut i = 0;
    for (enemy, zako, posture, zako_entity) in <(&mut Enemy, &mut Zako, &Posture, Entity)>::query().iter_mut(world) {
        for index in indices.iter() {
            if enemy.formation_index == *index && enemy.is_formation {
                let offset = &posture.0 - leader_pos;
                troops.members[i] = Some((*zako_entity, offset));
                i += 1;
                set_zako_to_troop(zako, enemy);
                break;
            }
        }
    }
    if i > 0 {
        commands.add_component(leader_entity, troops);
    }
}

pub fn update_troops(
    troops: &mut Troops, owner: Entity, _owl: &mut Owl, world: &mut SubWorld,
) {
    let &Posture(pos, angle) = <&Posture>::query().get(world, owner).unwrap();
    for member in troops.members.iter().flat_map(|x| x) {
        if let Ok(member_pos) = <&mut Posture>::query().get_mut(world, member.0) {
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

fn release_troops(troops: &mut Troops, world: &mut SubWorld) {
    for member_opt in troops.members.iter_mut().filter(|x| x.is_some()) {
        let member = member_opt.unwrap();
        if let Ok((enemy, zako_opt)) = <(&mut Enemy, Option<&mut Zako>)>::query().get_mut(world, member.0) {
            enemy.is_formation = true;
            if let Some(zako) = zako_opt {
                zako.state = ZakoState::Formation;
            }
        }
        *member_opt = None;
    }
}

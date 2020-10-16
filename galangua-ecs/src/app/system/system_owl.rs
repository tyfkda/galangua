use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::formation_table::X_COUNT;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::tractor_beam_table::*;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::traj_command::TrajCommand;
use galangua_common::app::game::traj_command_table::*;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::util::math::{atan2_lut, clamp, diff_angle, normalize_angle, ANGLE, ONE};

use crate::app::components::*;
use crate::app::resources::GameInfo;

use super::system_enemy::{forward, move_to_formation, set_zako_to_troop, update_traj};
use super::system_player::{
    escape_player_from_tractor_beam, move_capturing_player, set_player_captured,
    start_player_capturing, start_recapture_effect,
};

const LIFE: u32 = 2;

// Owl

pub fn create_owl(traj: Traj) -> Owl {
    Owl {
        state: OwlState::Appearance,
        traj: Some(traj),
        capturing_state: OwlCapturingState::None,
        target_pos: ZERO_VEC,
        //tractor_beam: None,
        count: 0,
        life: LIFE,
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
            let enemy = <&Enemy>::query().get(world, entity).unwrap();
            update_attack_traj(owl, enemy, posture, speed, formation, game_info);
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
                set_to_formation(owl, entity, world, commands);
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

fn update_attack_traj<'a>(owl: &mut Owl, enemy: &Enemy, posture: &mut Posture, speed: &mut Speed, formation: &Formation, game_info: &GameInfo) {
    let cont = if let Some(traj) = &mut owl.traj.as_mut() {
        update_traj(traj, posture, speed, formation)
    } else {
        false
    };
    if !cont {
        owl.traj = None;
        if game_info.is_rush() {
            // Rush mode: Continue attacking
            //self.remove_destroyed_troops(accessor);
            let table = &OWL_RUSH_ATTACK_TABLE;
            rush_attack(owl, table, posture, &enemy.formation_index);
            //accessor.push_event(EventType::PlaySe(CH_ATTACK, SE_ATTACK_START));
        } else {
            owl.state = OwlState::MoveToFormation;
        }
    }
}

fn rush_attack(owl: &mut Owl, table: &'static [TrajCommand], posture: &Posture, fi: &FormationIndex) {
    let flip_x = fi.0 >= 5;
    let mut traj = Traj::new(table, &ZERO_VEC, flip_x, *fi);
    traj.set_pos(&posture.0);

    //self.count = 0;
    //self.attack_frame_count = 0;
    owl.traj = Some(traj);
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
            let tractor_beam = tractor_beam_opt.unwrap();
            if tractor_beam_closed(tractor_beam) {
                //owl.tractor_beam = None;
                remove_tractor_beam(entity, commands);
                let spd = &mut speed.0;
                *spd = 5 * ONE / 2;

                owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::NoCaptureGoOut);
            } else if is_tractor_beam_capturing(tractor_beam) {
                game_info.capture_player();
                //accessor.push_event(EventType::PlaySe(CH_JINGLE, SE_TRACTOR_BEAM2));

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
            if owl.count == 0 {
                remove_tractor_beam(entity, commands);
            }

            owl.count += 1;
            if owl.count >= 120 {
                let spd = &mut speed.0;
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
            let done = {
                let troops = <&mut Troops>::query().get_mut(world, entity).unwrap();
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
                set_to_formation(owl, entity, world, commands);

                game_info.capture_completed();
            }
        }
    }
}

fn set_owl_capturing_player_completed(owl: &mut Owl, captured: bool) {
    if captured {
        owl.state = OwlState::CaptureAttack(OwlCaptureAttackPhase::CaptureDoneWait);
        owl.count = 0;
    }
}

fn set_to_formation(
    owl: &mut Owl,
    entity: Entity,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) -> bool {
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

    if owl.life == 0 {
        commands.remove(entity);
        false
    } else {
        true
    }
}

pub fn set_owl_damage(
    owl: &mut Owl, entity: Entity, power: u32,
    player_entity: Entity,
    attack_manager: &mut AttackManager,
    star_manager: &mut StarManager,
    game_info: &mut GameInfo,
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
) -> bool {
    if owl.life > power {
        //accessor.push_event(EventType::PlaySe(CH_BOMB, SE_DAMAGE));
        owl.life -= power;
        //DamageResult { point: 0, keep_alive_as_ghost: false }
        let drawable = <&mut SpriteDrawable>::query().get_mut(world, entity).unwrap();
        drawable.sprite_name = "cpp21";
        false
    } else {
        owl.life = 0;

        {
            let (mut subworld1, mut subworld2) = world.split::<&mut TractorBeam>();
            if let Ok(tractor_beam) = <&mut TractorBeam>::query().get_mut(&mut subworld1, entity) {
                break_tractor_beam(tractor_beam, &mut subworld2, commands);
                remove_tractor_beam(entity, commands);
            }
        }

        match owl.capturing_state {
            OwlCapturingState::None => {}
            OwlCapturingState::BeamTracting => {
                game_info.escape_capturing();
                star_manager.set_capturing(false);
            }
            OwlCapturingState::Attacking | OwlCapturingState::Failed => {
                game_info.end_capture_attack();
            }
        }

        let keep_alive_as_ghost = {
            let (mut subworld1, mut subworld2) = world.split::<&mut Troops>();
            if let Ok(troops) = <&mut Troops>::query().get_mut(&mut subworld1, entity) {
                let fi = <&Enemy>::query().get(&subworld2, entity).unwrap().formation_index;
                let cap_fi = FormationIndex(fi.0, fi.1 - 1);
                if let Some(slot) = troops.members.iter_mut().filter(|x| x.is_some()).find(|troop| {
                    if let Ok(enemy) = <&mut Enemy>::query().get_mut(&mut subworld2, troop.unwrap().0) {
                        enemy.formation_index == cap_fi
                    } else {
                        false
                    }
                }) {
                    let troop_entity = &slot.unwrap().0;
                    start_recapturing(*troop_entity, player_entity, attack_manager, game_info, &subworld2, commands);

                    *slot = None;
                }
                troops.members.iter().any(|x| x.is_some())
            } else {
                false
            }
        };

        owl.capturing_state = OwlCapturingState::None;

        if !keep_alive_as_ghost {
            commands.remove(entity);
        } else {
            // To keep moving troops.
            commands.remove_component::<SpriteDrawable>(entity);
            commands.remove_component::<CollRect>(entity);
        }
        true
    }
}

fn start_recapturing(
    owner_entity: Entity, player_entity: Entity,
    attack_manager: &mut AttackManager,
    game_info: &mut GameInfo,
    world: &SubWorld, commands: &mut CommandBuffer,
) {
    let &Posture(pos, angle) = <&Posture>::query().get(world, owner_entity).unwrap();
    start_recapture_effect(&pos, angle, player_entity, commands);
    commands.remove(owner_entity);
    attack_manager.pause(true);
    //system.play_se(CH_JINGLE, SE_RECAPTURE);

    game_info.start_recapturing();
    game_info.decrement_alive_enemy();
}

// Tractor Beam

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

fn remove_tractor_beam(entity: Entity, commands: &mut CommandBuffer) {
    commands.remove_component::<TractorBeam>(entity);
}

pub fn do_move_tractor_beam(
    tractor_beam: &mut TractorBeam, entity: Entity,
    owl: &mut Owl, enemy: &Enemy,
    game_info: &mut GameInfo,
    star_manager: &mut StarManager,
    attack_manager: &mut AttackManager,
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
                start_capturing(tractor_beam, player_entity);

                game_info.start_capturing();
                star_manager.set_capturing(true);
                attack_manager.pause(true);
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
                    tractor_beam.state = Closed;
                    let captured = tractor_beam.capturing_player.is_some();
                    if captured {
                        star_manager.set_capturing(false);
                    }
                    on_capturing_player_completed(owl, captured, game_info);
                    return;
                }
            }
        }
        Closed => {}
        Capturing => {
            let player_entity = tractor_beam.capturing_player.unwrap();
            let (player, posture) = <(&mut Player, &mut Posture)>::query().iter_mut(world).find(|_| true).unwrap();
            if move_capturing_player(player, posture, &(&tractor_beam.pos + &Vec2I::new(0, 8 * ONE))) {
                on_player_captured(
                    enemy, &tractor_beam.pos, entity, player_entity, game_info, commands);
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
    game_info: &mut GameInfo,
    commands: &mut CommandBuffer,
) {
    set_player_captured(player, commands);

    let fi = FormationIndex(enemy.formation_index.0, enemy.formation_index.1 - 1);
    let captured = commands.push((
        Enemy { enemy_type: EnemyType::CapturedFighter, formation_index: fi, is_formation: false },
        Zako { state: ZakoState::Troop, traj: None, target_pos: ZERO_VEC },
        Posture(pos + &Vec2I::new(0, 8 * ONE), 0),
        Speed(0, 0),
        CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) },
        SpriteDrawable {sprite_name: "rustacean_captured", offset: Vec2I::new(-8, -8)},
    ));

    let mut troops = Troops {members: Default::default()};
    add_captured_player_to_troops(&mut troops, captured, &Vec2I::new(0, 16 * ONE));
    commands.add_component(owner, troops);

    game_info.alive_enemy_count += 1;
}

fn on_capturing_player_completed(owl: &mut Owl, captured: bool, game_info: &mut GameInfo) {
    set_owl_capturing_player_completed(owl, captured);
    if captured {
        game_info.player_captured();
    }
}

fn tractor_beam_closed(tractor_beam: &TractorBeam) -> bool {
    tractor_beam.state == TractorBeamState::Closed
}

fn start_capturing(tractor_beam: &mut TractorBeam, player_entity: Entity) {
    tractor_beam.capturing_player = Some(player_entity);
    tractor_beam.state = TractorBeamState::Capturing;
    tractor_beam.count = 0;
}

fn is_tractor_beam_capturing(tractor_beam: &TractorBeam) -> bool {
    tractor_beam.state == TractorBeamState::Capturing
}

fn break_tractor_beam(tractor_beam: &mut TractorBeam, world: &mut SubWorld, commands: &mut CommandBuffer) {
    use TractorBeamState::*;

    for slot in tractor_beam.beam_sprites.iter_mut() {
        if let Some(beam) = slot {
            commands.remove(*beam);
            *slot = None;
        }
    }

    match tractor_beam.state {
        Capturing => {
            let player_entity = tractor_beam.capturing_player.unwrap();
            let player = <&mut Player>::query().get_mut(world, player_entity).unwrap();
            escape_player_from_tractor_beam(player);
        }
        _ => {
            assert!(tractor_beam.capturing_player.is_none());
        }
    }

    tractor_beam.state = Closed;
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

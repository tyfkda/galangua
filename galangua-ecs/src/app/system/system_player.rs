use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::framework::types::Vec2I;
use galangua_common::util::math::{calc_velocity, clamp, ANGLE, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;
use crate::app::resources::GameInfo;

const SPRITE_NAME: &str = "rustacean";
const HOME_X: i32 = (WIDTH / 2 - 8) * ONE;

pub fn new_player() -> Player {
    Player {
        state: PlayerState::Normal,
        count: 0,
        shot_enable: true,
        dual: None,
    }
}

pub fn do_move_player(
    player: &mut Player, pad: &Pad, entity: Entity,
    game_info: &mut GameInfo,
    attack_manager: &mut AttackManager,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) {
    use PlayerState::*;

    match player.state {
        Normal => {
            let x = {
                let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
                let pos = &mut posture.0;
                if pad.is_pressed(PadBit::L) {
                    pos.x -= PLAYER_SPEED;
                    let left = 8 * ONE;
                    if pos.x < left {
                        pos.x = left;
                    }
                }
                if pad.is_pressed(PadBit::R) {
                    pos.x += PLAYER_SPEED;
                    let right = if player.dual.is_some() { (WIDTH - 8 - 16) * ONE } else { (WIDTH - 8) * ONE };
                    if pos.x > right {
                        pos.x = right;
                    }
                }
                pos.x
            };

            if let Some(dual) = player.dual {
                let posture = <&mut Posture>::query().get_mut(world, dual).unwrap();
                posture.0.x = x + 16 * ONE;
            }
        }
        Capturing => {
            // Controled by TractorBeam, so nothing to do here.
        }
        Dead | Captured => {}
        MoveHomePos => {
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            let pos = &mut posture.0;
            let speed = 2 * ONE;
            pos.x += clamp(HOME_X - pos.x, -speed, speed);
        }
        EscapeCapturing => {
            const D: i32 = 1 * ONE;
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            let pos = &mut posture.0;
            let angle = &mut posture.1;
            pos.y += D;
            *angle = 0;
            if pos.y >= PLAYER_Y {
                pos.y = PLAYER_Y;
                player.state = Normal;
                commands.push((entity, player_coll_rect()));

                attack_manager.pause(false);
                game_info.escape_ended();
            }
        }
    }
}

fn set_player_recapture_done(
    player: &mut Player, entity: Entity,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) -> bool {
    let dual = player.state != PlayerState::Dead;
    if dual {
        let posture = <&Posture>::query().get(world, entity).unwrap();
        let dual = commands.push((
            Posture(&posture.0 + &Vec2I::new(16 * ONE, 0), 0),
            player_sprite(),
        ));
        player.dual = Some(dual);
    } else {
        let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
        restart_player(player, entity, posture, commands);
    }
    player.state = PlayerState::Normal;
    dual
}

pub fn move_capturing_player(player: &mut Player, posture: &mut Posture, target_pos: &Vec2I) -> bool {
    assert!(player.state == PlayerState::Capturing);
    const D: i32 = 1 * ONE;
    let pos = &mut posture.0;
    let angle = &mut posture.1;
    let d = target_pos - pos;
    pos.x += clamp(d.x, -D, D);
    pos.y += clamp(d.y, -D, D);
    *angle += ANGLE * ONE / ANGLE_DIV;

    //self.fire_bullet(pad, accessor);

    if d.x == 0 && d.y == 0 {
        player.state = PlayerState::Captured;
        *angle = 0;
        true
    } else {
        false
    }
}

pub fn can_player_fire(player: &Player) -> bool {
    if !player.shot_enable {
        return false;
    }
    match player.state {
        PlayerState::Normal | PlayerState::Capturing => true,
        _ => false
    }
}

pub fn crash_player(
    player: &mut Player, dual: bool, entity: Entity,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) -> bool {
    if dual {
        if let Some(dual) = player.dual.take() {
            commands.remove(dual);
        }
        false
    } else {
        if let Some(dual) = player.dual.take() {
            let dual_pos = <&Posture>::query().get_mut(world, dual).unwrap().0.clone();
            commands.remove(dual);
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            posture.0 = dual_pos;
            false
        } else {
            player.state = PlayerState::Dead;
            player.count = 0;
            commands.remove_component::<CollRect>(entity);
            commands.remove_component::<SpriteDrawable>(entity);
            true
        }
    }
}

pub fn start_player_capturing(player: &mut Player, entity: Entity, commands: &mut CommandBuffer) {
    player.state = PlayerState::Capturing;
    commands.remove_component::<CollRect>(entity);
}

pub fn set_player_captured(entity: Entity, commands: &mut CommandBuffer) {
    commands.remove_component::<SpriteDrawable>(entity);
}

pub fn escape_player_from_tractor_beam(player: &mut Player) {
    player.state = PlayerState::EscapeCapturing;
}

pub fn restart_player(player: &mut Player, entity: Entity, posture: &mut Posture, commands: &mut CommandBuffer) {
    player.state = PlayerState::Normal;
    posture.0 = Vec2I::new(CENTER_X, PLAYER_Y);

    commands.add_component(entity, player_sprite());
    commands.add_component(entity, player_coll_rect());
}

pub fn enable_player_shot(player: &mut Player, enable: bool) {
    player.shot_enable = enable;
}

pub fn player_sprite() -> SpriteDrawable {
    SpriteDrawable { sprite_name: SPRITE_NAME, offset: Vec2I::new(-8, -8) }
}

pub fn player_coll_rect() -> CollRect {
    CollRect { offset: Vec2I::new(-4, -4), size: Vec2I::new(8, 8) }
}

// MyShot

pub fn do_fire_myshot(player: &Player, posture: &Posture, entity: Entity, commands: &mut CommandBuffer) -> bool {
    if !can_player_fire(player) {
        return false;
    }

    let posture = Posture(&posture.0 + &calc_velocity(posture.1, 4 * ONE), posture.1);
    let dual = if player.dual.is_some() {
        assert!(posture.1 == 0);
        let second = commands.push((
            Posture(&posture.0 + &Vec2I::new(16 * ONE, 0), posture.1),
            SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)},
        ));
        Some(second)
    } else {
        None
    };

    commands.push((
        MyShot { player_entity: entity, dual },
        posture,
        CollRect { offset: Vec2I::new(-1, -4), size: Vec2I::new(1, 8) },
        SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)},
    ));
    true
}

pub fn do_move_myshot(shot: &MyShot, entity: Entity, world: &mut SubWorld, commands: &mut CommandBuffer) {
    let mut cont = false;
    for e in [Some(entity), shot.dual].iter().flat_map(|x| x) {
        let posture = <&mut Posture>::query().get_mut(world, *e).unwrap();
        let pos = &mut posture.0;
        let angle = &posture.1;

        if *angle == 0 {
            pos.y -= MYSHOT_SPEED;
        } else {
            *pos += &calc_velocity(*angle, MYSHOT_SPEED);
        }
        if !out_of_screen(pos) {
            cont = true;
        }
    }
    if !cont {
        delete_myshot(shot, entity, commands);
    }
}

fn out_of_screen(pos: &Vec2I) -> bool {
    const MARGIN: i32 = 4;
    const TOP: i32 = -MARGIN * ONE;
    const LEFT: i32 = -MARGIN * ONE;
    const RIGHT: i32 = (WIDTH + MARGIN) * ONE;
    const BOTTOM: i32 = (HEIGHT + MARGIN) * ONE;
    pos.y < TOP || pos.x < LEFT || pos.x > RIGHT || pos.y > BOTTOM
}

pub fn delete_myshot(shot: &MyShot, entity: Entity, commands: &mut CommandBuffer) {
    if let Some(dual) = shot.dual {
        commands.remove(dual);
    }
    commands.remove(entity);
}

// Recapture effect

pub fn start_recapture_effect(
    pos: &Vec2I, angle: i32, player_entity: Entity,
    commands: &mut CommandBuffer,
) {
    commands.push((
        RecapturedFighter { state: RecapturedFighterState::Rotate, count: 0, player_entity },
        Posture(*pos, angle),
        player_sprite(),
    ));
}

pub fn do_recapture_fighter(
    me: &mut RecapturedFighter, entity: Entity,
    attack_manager: &mut AttackManager,
    game_info: &mut GameInfo,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) {
    use RecapturedFighterState::*;
    const DANGLE: i32 = ANGLE * ONE / ANGLE_DIV;
    const SPEED: i32 = 2 * ONE;

    match me.state {
        Rotate => {
            let done = {
                let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
                let angle = &mut posture.1;
                *angle += DANGLE;
                if *angle >= ANGLE * ONE * 4 && attack_manager.is_no_attacker() {
                    *angle = 0;
                    true
                } else {
                    false
                }
            };
            if done {
                me.state = SlideHorz;
                //accessor.push_event(EventType::MovePlayerHomePos);

                let player = <&mut Player>::query().get_mut(world, me.player_entity).unwrap();
                if player.state != PlayerState::Dead {
                    player.state = PlayerState::MoveHomePos;
                }
            }
        }
        SlideHorz => {
            let player_living = <&Player>::query().get(world, me.player_entity).unwrap().state != PlayerState::Dead;
            let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
            let pos = &mut posture.0;
            let x = CENTER_X + if player_living { 8 * ONE } else { 0 };
            pos.x += clamp(x - pos.x, -SPEED, SPEED);
            if pos.x == x {
                me.state = SlideDown;
            }
        }
        SlideDown => {
            let done = {
                let posture = <&mut Posture>::query().get_mut(world, entity).unwrap();
                let pos = &mut posture.0;
                pos.y += clamp(PLAYER_Y - pos.y, -SPEED, SPEED);
                pos.y == PLAYER_Y
            };
            if done {
                let (mut subworld1, mut subworld2) = world.split::<&mut Player>();
                let player = <&mut Player>::query().get_mut(&mut subworld1, me.player_entity).unwrap();
                let dual = set_player_recapture_done(player, me.player_entity, &mut subworld2, commands);
                //accessor.push_event(EventType::RecaptureEnded(true));
                game_info.end_recapturing(dual);
                attack_manager.pause(false);
                commands.remove(entity);

                me.state = Done;
            }
        }
        Done => {}
    }
}

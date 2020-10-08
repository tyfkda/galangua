use specs::prelude::*;

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

pub fn move_player<'a>(
    player: &mut Player, entity: Entity, pad: &Pad,
    pos_storage: &mut WriteStorage<'a, Posture>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    game_info: &mut GameInfo,
    attack_manager: &mut AttackManager,
) {
    use PlayerState::*;

    match player.state {
        Normal => {
            let posture = pos_storage.get_mut(entity).unwrap();
            let x = {
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
                let posture = pos_storage.get_mut(dual).unwrap();
                posture.0.x = x + 16 * ONE;
            }
        }
        Capturing => {
            // Controled by TractorBeam, so nothing to do here.
        }
        Dead | Captured => {}
        MoveHomePos => {
            let posture = pos_storage.get_mut(entity).unwrap();
            let pos = &mut posture.0;
            let speed = 2 * ONE;
            pos.x += clamp(HOME_X - pos.x, -speed, speed);
        }
        EscapeCapturing => {
            const D: i32 = 1 * ONE;
            let posture = pos_storage.get_mut(entity).unwrap();
            let pos = &mut posture.0;
            let angle = &mut posture.1;
            pos.y += D;
            *angle = 0;
            if pos.y >= PLAYER_Y {
                pos.y = PLAYER_Y;
                player.state = Normal;
                coll_rect_storage.insert(entity, player_coll_rect()).unwrap();

                attack_manager.pause(false);
                game_info.escape_ended();
            }
        }
    }
}

fn set_player_recapture_done<'a>(
    player: &mut Player, entity: Entity, entities: &Entities,
    pos_storage: &mut WriteStorage<'a, Posture>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
) -> bool {
    let dual = player.state != PlayerState::Dead;
    if dual {
        let pos = pos_storage.get(entity).unwrap().0.clone();
        let dual = entities.build_entity()
            .with(Posture(&pos + &Vec2I::new(16 * ONE, 0), 0), pos_storage)
            .with(player_sprite(), drawable_storage)
            .build();
        player.dual = Some(dual);
    } else {
        let posture = pos_storage.get_mut(entity).unwrap();
        restart_player(player, entity, posture, drawable_storage, coll_rect_storage);
    }
    player.state = PlayerState::Normal;
    dual
}

pub fn move_capturing_player<'a>(player: &mut Player, posture: &mut Posture, target_pos: &Vec2I) -> bool {
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

pub fn crash_player<'a>(
    player: &mut Player, dual: bool, entity: Entity,
    pos_storage: &mut WriteStorage<'a, Posture>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    entities: &Entities<'a>,
) -> bool {
    if dual {
        if let Some(dual) = player.dual.take() {
            entities.delete(dual).unwrap();
        }
        false
    } else {
        if let Some(dual) = player.dual.take() {
            let dual_pos = pos_storage.get(dual).unwrap().0.clone();
            entities.delete(dual).unwrap();
            pos_storage.get_mut(entity).unwrap().0 = dual_pos;
            false
        } else {
            player.state = PlayerState::Dead;
            player.count = 0;
            drawable_storage.remove(entity);
            coll_rect_storage.remove(entity);
            true
        }
    }
}

pub fn start_player_capturing<'a>(player: &mut Player, entity: Entity, coll_rect_storage: &mut WriteStorage<'a, CollRect>) {
    player.state = PlayerState::Capturing;
    coll_rect_storage.remove(entity);
}

pub fn set_player_captured<'a>(entity: Entity, sprite_storage: &mut WriteStorage<'a, SpriteDrawable>) {
    sprite_storage.remove(entity);
}

pub fn escape_player_from_tractor_beam(player: &mut Player) {
    player.state = PlayerState::EscapeCapturing;
}

pub fn restart_player<'a>(player: &mut Player, entity: Entity, posture: &mut Posture, drawable_storage: &mut WriteStorage<'a, SpriteDrawable>, coll_rect_storage: &mut WriteStorage<'a, CollRect>) {
    player.state = PlayerState::Normal;
    posture.0 = Vec2I::new(CENTER_X, PLAYER_Y);

    drawable_storage.insert(entity, player_sprite()).unwrap();
    coll_rect_storage.insert(entity, player_coll_rect()).unwrap();
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

pub fn fire_myshot<'a>(player: &Player, entity: Entity, shot_storage: &mut WriteStorage<'a, MyShot>, pos_storage: &mut WriteStorage<'a, Posture>, coll_rect_storage: &mut WriteStorage<'a, CollRect>, drawable_storage: &mut WriteStorage<'a, SpriteDrawable>, entities: &Entities<'a>) -> bool {
    if !can_player_fire(player) {
        return false;
    }

    let mut posture = pos_storage.get(entity).unwrap().clone();
    posture.0 += &calc_velocity(posture.1, 4 * ONE);
    let dual = if player.dual.is_some() {
        assert!(posture.1 == 0);
        let second = entities.build_entity()
            .with(Posture(&posture.0 + &Vec2I::new(16 * ONE, 0), posture.1), pos_storage)
            .with(SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)}, drawable_storage)
            .build();
        Some(second)
    } else {
        None
    };
    entities.build_entity()
        .with(MyShot { player_entity: entity, dual }, shot_storage)
        .with(posture, pos_storage)
        .with(CollRect { offset: Vec2I::new(-1, -4), size: Vec2I::new(1, 8) }, coll_rect_storage)
        .with(SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)}, drawable_storage)
        .build();
    true
}

pub fn move_myshot<'a>(shot: &mut MyShot, entity: Entity, pos_storage: &mut WriteStorage<'a, Posture>, entities: &Entities<'a>) {
    let mut cont = false;
    for e in [Some(entity), shot.dual].iter().flat_map(|x| x) {
        let posture = pos_storage.get_mut(*e).unwrap();
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
        delete_myshot(shot, entity, &entities);
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

pub fn delete_myshot<'a>(shot: &MyShot, entity: Entity, entities: &Entities<'a>) {
    if let Some(dual) = shot.dual {
        entities.delete(dual).unwrap();
    }
    entities.delete(entity).unwrap();
}

// Recapture effect

pub fn start_recapture_effect<'a>(
    pos: &Vec2I, angle: i32, entities: &Entities<'a>, player_entity: Entity,
    recaptured_fighter_storage: &mut WriteStorage<'a, RecapturedFighter>,
    pos_storage: &mut WriteStorage<'a, Posture>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
) {
    entities.build_entity()
        .with(RecapturedFighter { state: RecapturedFighterState::Rotate, count: 0, player_entity }, recaptured_fighter_storage)
        .with(Posture(*pos, angle), pos_storage)
        .with(player_sprite(), drawable_storage)
        .build();
}

pub fn recapture_fighter<'a>(
    me: &mut RecapturedFighter, entity: Entity, entities: &Entities,
    player_storage: &mut WriteStorage<'a, Player>,
    pos_storage: &mut WriteStorage<'a, Posture>,
    coll_rect_storage: &mut WriteStorage<'a, CollRect>,
    drawable_storage: &mut WriteStorage<'a, SpriteDrawable>,
    attack_manager: &mut AttackManager,
    game_info: &mut GameInfo,
) {
    use RecapturedFighterState::*;
    const DANGLE: i32 = ANGLE * ONE / ANGLE_DIV;
    const SPEED: i32 = 2 * ONE;

    match me.state {
        Rotate => {
            let posture = pos_storage.get_mut(entity).unwrap();
            let angle = &mut posture.1;
            *angle += DANGLE;
            if *angle >= ANGLE * ONE * 4 && attack_manager.is_no_attacker() {
                me.state = SlideHorz;
                *angle = 0;
                //accessor.push_event(EventType::MovePlayerHomePos);

                let player = player_storage.get_mut(me.player_entity).unwrap();
                if player.state != PlayerState::Dead {
                    player.state = PlayerState::MoveHomePos;
                }
            }
        }
        SlideHorz => {
            let player = player_storage.get(me.player_entity).unwrap();
            let player_living = player.state != PlayerState::Dead;
            let posture = pos_storage.get_mut(entity).unwrap();
            let pos = &mut posture.0;
            let x = CENTER_X + if player_living { 8 * ONE } else { 0 };
            pos.x += clamp(x - pos.x, -SPEED, SPEED);
            if pos.x == x {
                me.state = SlideDown;
            }
        }
        SlideDown => {
            let posture = pos_storage.get_mut(entity).unwrap();
            let pos = &mut posture.0;
            pos.y += clamp(PLAYER_Y - pos.y, -SPEED, SPEED);
            if pos.y == PLAYER_Y {
                let player = player_storage.get_mut(me.player_entity).unwrap();
                let dual = set_player_recapture_done(player, me.player_entity, entities, pos_storage, coll_rect_storage, drawable_storage);
                //accessor.push_event(EventType::RecaptureEnded(true));
                game_info.end_recapturing(dual);
                attack_manager.pause(false);
                entities.delete(entity).unwrap();

                me.state = Done;
            }
        }
        Done => {}
    }
}

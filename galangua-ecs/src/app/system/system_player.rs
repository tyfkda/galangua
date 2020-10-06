use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::framework::types::Vec2I;
use galangua_common::util::math::{clamp, ANGLE, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;

pub fn new_player() -> Player {
    Player {
        state: PlayerState::Normal,
        count: 0,
        shot_enable: true,
    }
}

pub fn move_player<'a>(player: &mut Player, pad: &Pad, posture: &mut Posture) {
    match player.state {
        PlayerState::Normal => {
            let mut pos = &mut posture.0;
            if pad.is_pressed(PadBit::L) {
                pos.x -= PLAYER_SPEED;
            }
            if pad.is_pressed(PadBit::R) {
                pos.x += PLAYER_SPEED;
            }
            if pos.x < 8 * ONE {
                pos.x = 8 * ONE;
            } else if pos.x > (WIDTH - 8) * ONE {
                pos.x = (WIDTH - 8) * ONE;
            }
        }
        PlayerState::Capturing => {
            // Controled by TractorBeam, so nothing to do here.
        }
        PlayerState::Dead | PlayerState::Captured => {}
    }
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

pub fn crash_player<'a>(player: &mut Player, entity: Entity, drawable_storage: &mut WriteStorage<'a, SpriteDrawable>, coll_rect_storage: &mut WriteStorage<'a, CollRect>) -> bool {
    player.state = PlayerState::Dead;
    player.count = 0;
    drawable_storage.remove(entity);
    coll_rect_storage.remove(entity);
    true
}

pub fn start_player_capturing<'a>(player: &mut Player, entity: Entity, coll_rect_storage: &mut WriteStorage<'a, CollRect>) {
    player.state = PlayerState::Capturing;
    coll_rect_storage.remove(entity);
}

pub fn set_player_captured<'a>(entity: Entity, sprite_storage: &mut WriteStorage<'a, SpriteDrawable>) {
    sprite_storage.remove(entity);
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
    SpriteDrawable { sprite_name: "rustacean", offset: Vec2I::new(-8, -8) }
}

pub fn player_coll_rect() -> CollRect {
    CollRect { offset: Vec2I::new(-4, -4), size: Vec2I::new(8, 8) }
}

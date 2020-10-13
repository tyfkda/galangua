use legion::*;
use legion::systems::CommandBuffer;

use galangua_common::app::consts::*;
use galangua_common::framework::types::Vec2I;
use galangua_common::util::math::{clamp, ANGLE, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;

pub fn new_player() -> Player {
    Player {
        state: PlayerState::Normal,
        count: 0,
    }
}

pub fn do_move_player(player: &mut Player, entity: Entity, pad: &Pad, posture: &mut Posture, commands: &mut CommandBuffer) {
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
        PlayerState::Dead => {
            player.count += 1;
            if player.count >= 60 * 3 {
                commands.add_component(entity, player_coll_rect());
                commands.add_component(entity, player_sprite());
                player.state = PlayerState::Normal;
            }
        }
        PlayerState::Capturing => {
            // Controled by TractorBeam, so nothing to do here.
        }
        PlayerState::Captured => {}
    }
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
    match player.state {
        PlayerState::Normal | PlayerState::Capturing => true,
        _ => false
    }
}

pub fn crash_player(player: &mut Player, entity: Entity, commands: &mut CommandBuffer) {
    player.state = PlayerState::Dead;
    player.count = 0;
    commands.remove_component::<CollRect>(entity);
    commands.remove_component::<SpriteDrawable>(entity);
}

pub fn start_player_capturing(player: &mut Player, entity: Entity, commands: &mut CommandBuffer) {
    player.state = PlayerState::Capturing;
    commands.remove_component::<CollRect>(entity);
}

pub fn set_player_captured(entity: Entity, commands: &mut CommandBuffer) {
    commands.remove_component::<SpriteDrawable>(entity);
}

pub fn player_sprite() -> SpriteDrawable {
    SpriteDrawable { sprite_name: "rustacean", offset: Vec2I::new(-8, -8) }
}

pub fn player_coll_rect() -> CollRect {
    CollRect { offset: Vec2I::new(-4, -4), size: Vec2I::new(8, 8) }
}

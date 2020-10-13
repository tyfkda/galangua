use legion::*;
use legion::systems::CommandBuffer;

use galangua_common::app::consts::*;
use galangua_common::framework::types::Vec2I;
use galangua_common::util::math::ONE;
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
    }
}

pub fn crash_player(player: &mut Player, entity: Entity, commands: &mut CommandBuffer) {
    player.state = PlayerState::Dead;
    player.count = 0;
    commands.remove_component::<CollRect>(entity);
    commands.remove_component::<SpriteDrawable>(entity);
}

pub fn player_sprite() -> SpriteDrawable {
    SpriteDrawable { sprite_name: "rustacean", offset: Vec2I::new(-8, -8) }
}

pub fn player_coll_rect() -> CollRect {
    CollRect { offset: Vec2I::new(-4, -4), size: Vec2I::new(8, 8) }
}

use legion::*;

use galangua_common::app::consts::*;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{round_vec, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use super::components::*;

#[system]
pub fn update_pad(#[resource] pad: &mut Pad) {
    pad.update();
}

#[system(for_each)]
pub fn move_player(_player: &Player, pos: &mut Pos, #[resource] pad: &Pad) {
    let mut pos = &mut pos.0;
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

pub fn draw_system<R: RendererTrait>(world: &World, renderer: &mut R) {
    let mut query = <(&Pos, &SpriteDrawable)>::query();
    for (pos, drawable) in query.iter(world) {
        let pos = round_vec(&pos.0);
        renderer.draw_sprite(drawable.sprite_name, &(&pos + &drawable.offset));
    }
}

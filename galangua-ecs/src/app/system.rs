use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::framework::types::Vec2I;
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

#[system(for_each)]
#[write_component(MyShot)]
pub fn fire_myshot(_player: &Player, pos: &Pos, world: &mut SubWorld, #[resource] pad: &Pad, commands: &mut CommandBuffer) {
    let shot_count = <&MyShot>::query().iter(world).count();
    if pad.is_trigger(PadBit::A) {
        if shot_count < 2 {
            commands.push((
                MyShot,
                Pos(pos.0.clone()),
                SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)},
            ));
        }
    }
}

#[system(for_each)]
pub fn move_myshot(_myshot: &MyShot, pos: &mut Pos, entity: &Entity, commands: &mut CommandBuffer) {
    let mut pos = &mut pos.0;
    pos.y -= MYSHOT_SPEED;
    if pos.y < 0 * ONE {
        commands.remove(*entity);

    }
}

pub fn draw_system<R: RendererTrait>(world: &World, renderer: &mut R) {
    let mut query = <(&Pos, &SpriteDrawable)>::query();
    for (pos, drawable) in query.iter(world) {
        let pos = round_vec(&pos.0);
        renderer.draw_sprite(drawable.sprite_name, &(&pos + &drawable.offset));
    }
}

use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{round_vec, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use super::components::*;

pub struct SysPadUpdater;
impl<'a> System<'a> for SysPadUpdater {
    type SystemData = Write<'a, Pad>;

    fn run(&mut self, mut pad: Self::SystemData) {
        pad.update();
    }
}

pub struct SysPlayerMover;
impl<'a> System<'a> for SysPlayerMover {
    type SystemData = (Read<'a, Pad>, ReadStorage<'a, Player>, WriteStorage<'a, Pos>);

    fn run(&mut self, (pad, player_storage, mut pos_storage): Self::SystemData) {
        for (_player, pos) in (&player_storage, &mut pos_storage).join() {
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
    }
}

pub struct SysDrawer<'a>(pub &'a mut dyn RendererTrait);
impl<'a> System<'a> for SysDrawer<'a> {
    type SystemData = (ReadStorage<'a, Pos>, ReadStorage<'a, SpriteDrawable>);

    fn run(&mut self, (pos_storage, drawable_storage): Self::SystemData) {
        let renderer = &mut self.0;
        for (pos, drawable) in (&pos_storage, &drawable_storage).join() {
            let pos = round_vec(&pos.0);
            renderer.draw_sprite(drawable.sprite_name, &(&pos + &drawable.offset));
        }
    }
}

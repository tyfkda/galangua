use specs::prelude::*;

use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{round_vec, ONE};

use super::components::*;

pub struct SysMover;
impl<'a> System<'a> for SysMover {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos_storage, vel_storage): Self::SystemData) {
        for (pos, vel) in (&mut pos_storage, &vel_storage).join() {
            pos.0 += &vel.0;
            if pos.0.x > 224 * ONE {
                pos.0.x = 0;
            }
        }
    }
}

pub struct SysDrawer<'a>(pub &'a mut dyn RendererTrait);
impl<'a> System<'a> for SysDrawer<'a> {
    type SystemData = (ReadStorage<'a, Pos>, ReadStorage<'a, SpriteDrawable>);

    fn run(&mut self, (pos_storage, drawable_storage): Self::SystemData) {
        let renderer = &mut self.0;
        for (pos, _drawable) in (&pos_storage, &drawable_storage).join() {
            let pos = round_vec(&pos.0);
            renderer.draw_sprite("gopher1", &pos);
        }
    }
}

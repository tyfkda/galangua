use legion::*;

use galangua_common::framework::RendererTrait;

use galangua_common::util::math::{round_vec, ONE};

use super::components::*;

// systems are normal Rust fns, annotated with #[system]
#[system(for_each)]
pub fn mover(pos: &mut Pos, vel: &Vel) {
    pos.0 += &vel.0;
    if pos.0.x > 224 * ONE {
        pos.0.x = 0;
    }
}

pub fn draw_system<R: RendererTrait>(world: &World, renderer: &mut R) {
    let mut query = <(&Pos, &SpriteDrawable)>::query();
    for (pos, _drawable) in query.iter(world) {
        let pos = round_vec(&pos.0);
        renderer.draw_sprite("gopher1", &pos);
    }
}

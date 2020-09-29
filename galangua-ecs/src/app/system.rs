use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::framework::types::Vec2I;
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

pub struct SysPlayerFirer;
impl<'a> System<'a> for SysPlayerFirer {
    type SystemData = (
        Read<'a, Pad>,
        ReadStorage<'a, Player>,
        Entities<'a>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, MyShot>,
        WriteStorage<'a, SpriteDrawable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (pad,
             player_storage,
             entities,
             mut pos_storage,
             mut shot_storage,
             mut drawable_storage) = data;

        let mut shot_count = 0;
        for _shot in shot_storage.join() {
            shot_count += 1;
        }

        let shots = (&player_storage, &mut pos_storage).join()
            .flat_map(|(_player, pos)| {
                if pad.is_trigger(PadBit::A) {
                    Some(pos.0.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Vec2I>>();

        shots.iter().for_each(|pos| {
            if shot_count < 2 {
                entities.build_entity()
                    .with(MyShot, &mut shot_storage)
                    .with(Pos(*pos), &mut pos_storage)
                    .with(SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)}, &mut drawable_storage)
                    .build();
                shot_count += 1;
            }
        })
    }
}

pub struct SysMyShotMover;
impl<'a> System<'a> for SysMyShotMover {
    type SystemData = (ReadStorage<'a, MyShot>, WriteStorage<'a, Pos>, Entities<'a>);

    fn run(&mut self, (shot_storage, mut pos_storage, entities): Self::SystemData) {
        for (_shot, pos, entity) in (&shot_storage, &mut pos_storage, &*entities).join() {
            let mut pos = &mut pos.0;
            pos.y -= MYSHOT_SPEED;
            if pos.y < 0 * ONE {
                entities.delete(entity).unwrap();
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

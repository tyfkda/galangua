use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::util::collision::CollBox;
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
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SpriteDrawable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (pad,
             player_storage,
             entities,
             mut pos_storage,
             mut shot_storage,
             mut coll_rect_storage,
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
                    .with(CollRect { offset: Vec2I::new(-1, -4), size: Vec2I::new(1, 8) }, &mut coll_rect_storage)
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

pub struct SysFormationMover;
impl<'a> System<'a> for SysFormationMover {
    type SystemData = Write<'a, Formation>;

    fn run(&mut self, mut formation: Self::SystemData) {
        formation.update();
    }
}

pub struct SysEnemyMover;
impl<'a> System<'a> for SysEnemyMover {
    type SystemData = (ReadStorage<'a, Enemy>, Read<'a, Formation>, WriteStorage<'a, Pos>);

    fn run(&mut self, (enemy_storage, formation, mut pos_storage): Self::SystemData) {
        for (enemy, pos) in (&enemy_storage, &mut pos_storage).join() {
            *pos = Pos(formation.pos(&enemy.formation_index));
        }
    }
}

pub struct SysCollCheckMyShotEnemy;
impl<'a> System<'a> for SysCollCheckMyShotEnemy {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, MyShot>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, CollRect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             pos_storage,
             shot_storage,
             enemy_storage,
             coll_rect_storage) = data;

        for (_shot, shot_pos, shot_coll_rect, shot_entity) in (&shot_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
            let shot_collbox = CollBox { top_left: &round_vec(&shot_pos.0) + &shot_coll_rect.offset, size: shot_coll_rect.size };
            for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in (&enemy_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
                let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                if shot_collbox.check_collision(&enemy_collbox) {
                    entities.delete(enemy_entity).unwrap();
                    entities.delete(shot_entity).unwrap();
                    break;
                }
            }
        }
    }
}

pub struct SysStarMover;
impl<'a> System<'a> for SysStarMover {
    type SystemData = Write<'a, StarManager>;

    fn run(&mut self, mut star_manager: Self::SystemData) {
        star_manager.update();
    }
}

pub struct SysDrawer<'a, R: RendererTrait>(pub &'a mut R);
impl<'a, R: RendererTrait> System<'a> for SysDrawer<'a, R> {
    type SystemData = (Read<'a, StarManager>, ReadStorage<'a, Pos>, ReadStorage<'a, SpriteDrawable>);

    fn run(&mut self, (star_manager, pos_storage, drawable_storage): Self::SystemData) {
        let renderer = &mut self.0;

        star_manager.draw(*renderer);

        for (pos, drawable) in (&pos_storage, &drawable_storage).join() {
            let pos = round_vec(&pos.0);
            renderer.draw_sprite(drawable.sprite_name, &(&pos + &drawable.offset));
        }
    }
}

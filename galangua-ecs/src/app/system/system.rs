use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::appearance_manager::Accessor as AppearanceManagerAccessor;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::attack_manager::Accessor as AttackManagerAccessor;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{quantize_angle, round_vec, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;

use super::system_effect::*;
use super::system_enemy::*;
use super::system_player::*;

pub struct SysPadUpdater;
impl<'a> System<'a> for SysPadUpdater {
    type SystemData = Write<'a, Pad>;

    fn run(&mut self, mut pad: Self::SystemData) {
        pad.update();
    }
}

pub struct SysPlayerMover;
impl<'a> System<'a> for SysPlayerMover {
    type SystemData = (Read<'a, Pad>, WriteStorage<'a, Player>, WriteStorage<'a, Posture>, WriteStorage<'a, SpriteDrawable>, WriteStorage<'a, CollRect>, Entities<'a>);

    fn run(&mut self, (pad, mut player_storage, mut pos_storage, mut sprite_storage, mut coll_rect_storage, entities): Self::SystemData) {
        for (player, mut posture, entity) in (&mut player_storage, &mut pos_storage, &*entities).join() {
            move_player(player, entity, &pad, &mut posture, &mut sprite_storage, &mut coll_rect_storage);
        }
    }
}

pub struct SysPlayerFirer;
impl<'a> System<'a> for SysPlayerFirer {
    type SystemData = (
        Read<'a, Pad>,
        ReadStorage<'a, Player>,
        Entities<'a>,
        WriteStorage<'a, Posture>,
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
                    Some(pos.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Posture>>();

        shots.into_iter().for_each(|pos| {
            if shot_count < 2 {
                entities.build_entity()
                    .with(MyShot, &mut shot_storage)
                    .with(pos, &mut pos_storage)
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
    type SystemData = (ReadStorage<'a, MyShot>, WriteStorage<'a, Posture>, Entities<'a>);

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

pub struct SysAppearanceManager;
impl<'a> System<'a> for SysAppearanceManager {
    type SystemData = (
        Write<'a, AppearanceManager>,
        Entities<'a>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Zako>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SpriteDrawable>,
        Write<'a, Formation>,
        Write<'a, AttackManager>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut appearance_manager,
            entities,
            mut enemy_storage,
            mut zako_storage,
            mut posture_storage,
            mut speed_storage,
            mut coll_rect_storage,
            mut drawable_storage,
            mut formation,
            mut attack_manager) = data;

        if appearance_manager.done {
            return;
        }

        let accessor = SysAppearanceManagerAccessor(&zako_storage);
        let new_borns_opt = appearance_manager.update(&accessor);
        if let Some(new_borns) = new_borns_opt {
            for e in new_borns {
                let sprite_name = match e.enemy_type {
                    EnemyType::Bee => "gopher1",
                    EnemyType::Butterfly => "dman1",
                    EnemyType::Owl => "cpp11",
                    EnemyType::CapturedFighter => "rustacean_captured",
                };
                entities.build_entity()
                    .with(Enemy { enemy_type: e.enemy_type, formation_index: e.fi }, &mut enemy_storage)
                    .with(Zako { state: ZakoState::Appearance, traj: Some(e.traj) }, &mut zako_storage)
                    .with(Posture(e.pos, 0), &mut posture_storage)
                    .with(Speed(0, 0), &mut speed_storage)
                    .with(CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) }, &mut coll_rect_storage)
                    .with(SpriteDrawable {sprite_name, offset: Vec2I::new(-8, -8)}, &mut drawable_storage)
                    .build();
            }
        }

        if appearance_manager.done {
            formation.done_appearance();
            attack_manager.set_enable(true);
        }
    }
}

struct SysAppearanceManagerAccessor<'a>(&'a WriteStorage<'a, Zako>);
impl<'a> AppearanceManagerAccessor for SysAppearanceManagerAccessor<'a> {
    fn is_stationary(&self) -> bool {
        let zako_storage = &self.0;
        zako_storage.join().all(|x| x.state == ZakoState::Formation)
    }
}

pub struct SysAttackManager;
impl<'a> System<'a> for SysAttackManager {
    type SystemData = (
        Write<'a, AttackManager>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Zako>,
        ReadStorage<'a, Posture>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut attack_manager,
             mut enemy_storage,
             mut zako_storage,
             posture_storage) = data;

        let result = {
            let accessor = SysAttackManagerAccessor(&enemy_storage, &zako_storage);
            attack_manager.update(&accessor)
        };
        if let Some((fi, capture_attack)) = result {
            (&mut enemy_storage, &mut zako_storage, &posture_storage).join()
                .filter(|(enemy, ..)| enemy.formation_index == fi)
                .for_each(|(enemy, zako, posture)| {
                    zako_start_attack(zako, &enemy, &posture, capture_attack);
                    attack_manager.put_attacker(&fi);
                });
        }
    }
}

struct SysAttackManagerAccessor<'a>(&'a WriteStorage<'a, Enemy>, &'a WriteStorage<'a, Zako>);
impl<'a> SysAttackManagerAccessor<'a> {
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&'a Enemy> {
        self.0.join()
            .find(|enemy| enemy.formation_index == *formation_index)
    }
    fn get_zako_at(&self, formation_index: &FormationIndex) -> Option<&'a Zako> {
        (self.0, self.1).join()
            .find(|(enemy, _zako)| enemy.formation_index == *formation_index)
            .map(|(_enemy, zako)| zako)
    }
}
impl<'a> AttackManagerAccessor for SysAttackManagerAccessor<'a> {
    fn can_capture_attack(&self) -> bool { true }
    fn captured_fighter_index(&self) -> Option<FormationIndex> { None }
    fn is_enemy_live_at(&self, formation_index: &FormationIndex) -> bool {
        self.get_enemy_at(formation_index).is_some()
    }
    fn is_enemy_formation_at(&self, formation_index: &FormationIndex) -> bool {
        if let Some(zako) = self.get_zako_at(formation_index) {
            zako.state == ZakoState::Formation
        } else {
            false
        }
    }
}

pub struct SysZakoMover;
impl<'a> System<'a> for SysZakoMover {
    type SystemData = (ReadStorage<'a, Enemy>, WriteStorage<'a, Zako>, Read<'a, Formation>, WriteStorage<'a, Posture>, WriteStorage<'a, Speed>);

    fn run(&mut self, (enemy_storage, mut zako_storage, formation, mut pos_storage, mut speed_storage): Self::SystemData) {
        for (enemy, zako, posture, vel) in (&enemy_storage, &mut zako_storage, &mut pos_storage, &mut speed_storage).join() {
            move_zako(zako, enemy, posture, vel, &formation);
        }
    }
}

pub struct SysCollCheckMyShotEnemy;
impl<'a> System<'a> for SysCollCheckMyShotEnemy {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Posture>,
        ReadStorage<'a, MyShot>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, CollRect>,
        WriteStorage<'a, SequentialSpriteAnime>,
        WriteStorage<'a, SpriteDrawable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut pos_storage,
             shot_storage,
             enemy_storage,
             coll_rect_storage,
             mut seqanime_storage,
             mut sprite_storage) = data;

        let mut colls: Vec<Vec2I> = Vec::new();
        for (_shot, shot_pos, shot_coll_rect, shot_entity) in (&shot_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
            let shot_collbox = CollBox { top_left: &round_vec(&shot_pos.0) + &shot_coll_rect.offset, size: shot_coll_rect.size };
            for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in (&enemy_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
                let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                if shot_collbox.check_collision(&enemy_collbox) {
                    entities.delete(enemy_entity).unwrap();
                    entities.delete(shot_entity).unwrap();
                    colls.push(enemy_pos.0.clone());
                    break;
                }
            }
        }

        for coll in colls.iter() {
            create_enemy_explosion_effect(coll, &entities, &mut pos_storage, &mut seqanime_storage, &mut sprite_storage);
        }
    }
}

pub struct SysCollCheckPlayerEnemy;
impl<'a> System<'a> for SysCollCheckPlayerEnemy {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, Enemy>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SequentialSpriteAnime>,
        WriteStorage<'a, SpriteDrawable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut pos_storage,
             mut player_storage,
             enemy_storage,
             mut coll_rect_storage,
             mut seqanime_storage,
             mut sprite_storage) = data;

        let mut colls: Vec<(Entity, Vec2I, Vec2I)> = Vec::new();
        for (_player, player_pos, player_coll_rect, player_entity) in (&player_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
            let player_collbox = CollBox { top_left: &round_vec(&player_pos.0) + &player_coll_rect.offset, size: player_coll_rect.size };
            for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in (&enemy_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
                let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                if player_collbox.check_collision(&enemy_collbox) {
                    entities.delete(enemy_entity).unwrap();
                    colls.push((player_entity, player_pos.0.clone(), enemy_pos.0.clone()));
                    break;
                }
            }
        }

        for (player_entity, player_pos, enemy_pos) in colls.iter() {
            create_player_explosion_effect(player_pos, &entities, &mut pos_storage, &mut seqanime_storage, &mut sprite_storage);
            create_enemy_explosion_effect(enemy_pos, &entities, &mut pos_storage, &mut seqanime_storage, &mut sprite_storage);

            let player = player_storage.get_mut(*player_entity).unwrap();
            crash_player(player, *player_entity, &mut sprite_storage, &mut coll_rect_storage);
        }
    }
}

pub struct SysSequentialSpriteAnime;
impl<'a> System<'a> for SysSequentialSpriteAnime {
    type SystemData = (WriteStorage<'a, SequentialSpriteAnime>, WriteStorage<'a, SpriteDrawable>, Entities<'a>);

    fn run(&mut self, (mut seqanime_storage, mut sprite_storage, entities): Self::SystemData) {
        for (anime, sprite, entity) in (&mut seqanime_storage, &mut sprite_storage, &*entities).join() {
            if let Some(sprite_name) = update_seqanime(anime) {
                sprite.sprite_name = sprite_name;
            } else {
                entities.delete(entity).unwrap();
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
    type SystemData = (Read<'a, StarManager>, ReadStorage<'a, Posture>, ReadStorage<'a, SpriteDrawable>);

    fn run(&mut self, (star_manager, pos_storage, drawable_storage): Self::SystemData) {
        let renderer = &mut self.0;

        star_manager.draw(*renderer);
        for (posture, drawable) in (&pos_storage, &drawable_storage).join() {
            let pos = &round_vec(&posture.0) + &drawable.offset;
            let angle = quantize_angle(posture.1, ANGLE_DIV);
            if angle == 0 {
                renderer.draw_sprite(drawable.sprite_name, &pos);
            } else {
                renderer.draw_sprite_rot(drawable.sprite_name, &pos, angle, None);
            }
        }
    }
}

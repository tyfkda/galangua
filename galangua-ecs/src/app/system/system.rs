use specs::prelude::*;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::appearance_manager::Accessor as AppearanceManagerAccessor;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::attack_manager::Accessor as AttackManagerAccessor;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::stage_indicator::StageIndicator;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::{CaptureState, EnemyType, FormationIndex};
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{quantize_angle, round_vec};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;
use crate::app::resources::*;

use super::system_effect::*;
use super::system_enemy::*;
use super::system_player::*;
use super::system_owl::*;

pub struct SysGameController;
impl<'a> System<'a> for SysGameController {
    type SystemData = (Write<'a, GameInfo>, GameInfoUpdateParams<'a>);

    fn run(&mut self, (mut game_info, params): Self::SystemData) {
        game_info.update(params);
    }
}

pub struct SysPlayerMover;
impl<'a> System<'a> for SysPlayerMover {
    type SystemData = (
        Read<'a, Pad>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, CollRect>,
        Entities<'a>,
        Write<'a, GameInfo>,
        Write<'a, AttackManager>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (pad,
             mut player_storage,
             mut pos_storage,
             mut coll_rect_storage,
             entities,
             mut game_info,
             mut attack_manager) = data;

        for (player, entity) in (&mut player_storage, &*entities).join() {
            move_player(
                player, entity, &pad, &mut pos_storage, &mut coll_rect_storage,
                &mut game_info, &mut attack_manager);
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

        for (player, entity) in (&player_storage, &*entities).join() {
            if pad.is_trigger(PadBit::A) && shot_count < 2 {
                if fire_myshot(player, entity, &mut shot_storage, &mut pos_storage, &mut coll_rect_storage, &mut drawable_storage, &entities) {
                    shot_count += 1;
                }
            }
        }
    }
}

pub struct SysMyShotMover;
impl<'a> System<'a> for SysMyShotMover {
    type SystemData = (WriteStorage<'a, MyShot>, WriteStorage<'a, Posture>, Entities<'a>);

    fn run(&mut self, (mut shot_storage, mut pos_storage, entities): Self::SystemData) {
        for (shot, entity) in (&mut shot_storage, &*entities).join() {
            move_myshot(shot, entity, &mut pos_storage, &entities);
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
        WriteStorage<'a, Owl>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SpriteDrawable>,
        Write<'a, Formation>,
        Write<'a, AttackManager>,
        Write<'a, GameInfo>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut appearance_manager,
            entities,
            mut enemy_storage,
            mut zako_storage,
            mut owl_storage,
            mut posture_storage,
            mut speed_storage,
            mut coll_rect_storage,
            mut drawable_storage,
            mut formation,
            mut attack_manager,
            mut game_info) = data;

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
                let mut builder = entities.build_entity()
                    .with(Enemy { enemy_type: e.enemy_type, formation_index: e.fi, is_formation: false }, &mut enemy_storage)
                    .with(Posture(e.pos, 0), &mut posture_storage)
                    .with(Speed(0, 0), &mut speed_storage)
                    .with(CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) }, &mut coll_rect_storage)
                    .with(SpriteDrawable {sprite_name, offset: Vec2I::new(-8, -8)}, &mut drawable_storage);
                builder = if e.enemy_type != EnemyType::Owl {
                    let base = EnemyBase::new(Some(e.traj));
                    builder.with(Zako { base, state: ZakoState::Appearance }, &mut zako_storage)
                } else {
                    builder.with(create_owl(e.traj), &mut owl_storage)
                };
                builder.build();
                game_info.alive_enemy_count += 1;
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
        Entities<'a>,
        Write<'a, AttackManager>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Zako>,
        WriteStorage<'a, Owl>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, Troops>,
        ReadStorage<'a, Player>,
        Write<'a, GameInfo>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut attack_manager,
             mut enemy_storage,
             mut zako_storage,
             mut owl_storage,
             mut pos_storage,
             mut speed_storage,
             mut troops_storage,
             player_storage,
             mut game_info) = data;

        let result = {
            let accessor = SysAttackManagerAccessor(&enemy_storage, &game_info);
            attack_manager.update(&accessor)
        };
        if let Some((fi, capture_attack)) = result {
            let get_player_pos = || {
                for (_player, posture) in (&player_storage, &pos_storage).join() {
                    return Some(posture.0.clone());
                }
                None
            };

            let player_pos = get_player_pos().unwrap();
            let entity = (&enemy_storage, &*entities).join()
                .find(|(enemy, _entity)| enemy.formation_index == fi)
                .map(|(_enemy, entity)| entity);

            if let Some(entity) = entity {
                let speed = speed_storage.get_mut(entity).unwrap();

                if let Some(zako) = zako_storage.get_mut(entity) {
                    let enemy = enemy_storage.get_mut(entity).unwrap();
                    let posture = pos_storage.get_mut(entity).unwrap();
                    zako_start_attack(zako, enemy, posture);
                }
                if let Some(owl) = owl_storage.get_mut(entity) {
                    owl_start_attack(
                        owl, entity, capture_attack, speed, &player_pos,
                        &mut enemy_storage, &mut zako_storage, &mut troops_storage,
                        &mut pos_storage, entities);
                    if capture_attack {
                        game_info.capture_state = CaptureState::CaptureAttacking;
                        game_info.capture_enemy_fi = fi;
                    }
                }
                attack_manager.put_attacker(&fi);
            }
        }
    }
}

struct SysAttackManagerAccessor<'a>(&'a WriteStorage<'a, Enemy>, &'a GameInfo);
impl<'a> SysAttackManagerAccessor<'a> {
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&'a Enemy> {
        self.0.join()
            .find(|enemy| enemy.formation_index == *formation_index)
    }
}
impl<'a> AttackManagerAccessor for SysAttackManagerAccessor<'a> {
    fn can_capture_attack(&self) -> bool { self.1.can_capture_attack() }
    fn captured_fighter_index(&self) -> Option<FormationIndex> {
        match self.1.capture_state {
            CaptureState::CaptureAttacking |
            CaptureState::Capturing |
            CaptureState::Captured => {
                Some(FormationIndex(self.1.capture_enemy_fi.0, self.1.capture_enemy_fi.1 - 1))
            }
            _ => { None }
        }
    }
    fn is_enemy_live_at(&self, formation_index: &FormationIndex) -> bool {
        self.get_enemy_at(formation_index).is_some()
    }
    fn is_enemy_formation_at(&self, formation_index: &FormationIndex) -> bool {
        if let Some(enemy) = self.get_enemy_at(formation_index) {
            enemy.is_formation
        } else {
            false
        }
    }
}

pub struct SysZakoMover;
impl<'a> System<'a> for SysZakoMover {
    type SystemData = (
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Zako>,
        Read<'a, Formation>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Speed>,
        ReadStorage<'a, Player>,
        Entities<'a>,
        Write<'a, GameInfo>,
        Write<'a, EneShotSpawner>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut enemy_storage,
             mut zako_storage,
             formation,
             mut pos_storage,
             mut speed_storage,
             player_storage,
             entities,
             mut game_info,
             mut eneshot_spawner) = data;

        for (enemy, zako, speed, entity) in (&mut enemy_storage, &mut zako_storage, &mut speed_storage, &*entities).join() {
            move_zako(zako, entity, enemy, speed, &formation, &player_storage, &mut pos_storage, &entities, &mut game_info, &mut eneshot_spawner);
        }
    }
}

pub struct SysOwlMover;
impl<'a> System<'a> for SysOwlMover {
    type SystemData = (
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Owl>,
        Read<'a, Formation>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Speed>,
        Entities<'a>,
        WriteStorage<'a, Zako>,
        WriteStorage<'a, TractorBeam>,
        WriteStorage<'a, Troops>,
        Write<'a, GameInfo>,
        Write<'a, EneShotSpawner>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut enemy_storage,
             mut owl_storage,
             formation,
             mut pos_storage,
             mut speed_storage,
             entities,
             mut zako_storage,
             mut tractor_beam_storage,
             mut troops_storage,
             mut game_info,
             mut eneshot_spawner) = data;

        for (owl, posture, speed, entity) in (&mut owl_storage, &mut pos_storage, &mut speed_storage, &*entities).join() {
            move_owl(
                owl, entity, posture, speed, &formation, &entities, &mut enemy_storage, &mut zako_storage, &mut tractor_beam_storage, &mut troops_storage, &mut game_info,
                &mut eneshot_spawner);
        }
    }
}

pub struct SysTroopsMover;
impl<'a> System<'a> for SysTroopsMover {
    type SystemData = (
        WriteStorage<'a, Troops>,
        Entities<'a>,
        ReadStorage<'a, Owl>,
        WriteStorage<'a, Posture>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut troops_storage,
             entities,
             owl_storage,
             mut pos_storage) = data;

        for (mut troops, entity) in (&mut troops_storage, &*entities).join() {
            update_troops(
                &mut troops, &entity, &owl_storage, &mut pos_storage);
        }
    }
}

pub struct SysTractorBeamMover;
impl<'a> System<'a> for SysTractorBeamMover {
    type SystemData = (
        WriteStorage<'a, TractorBeam>,
        Write<'a, GameInfo>,
        Entities<'a>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, SpriteDrawable>,
        WriteStorage<'a, SpriteColor>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Zako>,
        WriteStorage<'a, Owl>,
        WriteStorage<'a, Speed>,
        WriteStorage<'a, Troops>,
        Write<'a, StarManager>,
        Write<'a, AttackManager>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut tractor_beam_storage,
             mut game_info,
             entities,
             mut pos_storage,
             mut drawable_storage,
             mut color_storage,
             mut player_storage,
             mut coll_rect_storage,
             mut enemy_storage,
             mut zako_storage,
             mut owl_storage,
             mut speed_storage,
             mut troops_storage,
             mut star_manager,
             mut attack_manager) = data;

        for (tractor_beam, owl, entity) in (&mut tractor_beam_storage, &mut owl_storage, &*entities).join() {
            move_tractor_beam(
                tractor_beam, &mut game_info, &entities, entity, owl, &mut pos_storage,
                &mut drawable_storage, &mut color_storage, &mut player_storage, &mut coll_rect_storage,
                &mut enemy_storage, &mut zako_storage, &mut speed_storage, &mut troops_storage,
                &mut star_manager, &mut attack_manager);
        }
    }
}

pub struct SysEneShotSpawner;
impl<'a> System<'a> for SysEneShotSpawner {
    type SystemData = (Write<'a, EneShotSpawner>, EneShotSpawnerParam<'a>);

    fn run(&mut self, (mut eneshot_spawner, param): Self::SystemData) {
        eneshot_spawner.update(param);
    }
}

pub struct SysEneShotMover;
impl<'a> System<'a> for SysEneShotMover {
    type SystemData = (WriteStorage<'a, EneShot>, WriteStorage<'a, Posture>, Entities<'a>);

    fn run(&mut self, (mut shot_storage, mut pos_storage, entities): Self::SystemData) {
        for (shot, posture, entity) in (&mut shot_storage, &mut pos_storage, &*entities).join() {
            move_eneshot(shot, posture, entity, &entities);
        }
    }
}

pub struct SysCollCheckMyShotEnemy;
impl<'a> System<'a> for SysCollCheckMyShotEnemy {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, MyShot>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SequentialSpriteAnime>,
        WriteStorage<'a, SpriteDrawable>,
        ReadStorage<'a, Zako>,
        WriteStorage<'a, Owl>,
        WriteStorage<'a, Troops>,
        WriteStorage<'a, RecapturedFighter>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, TractorBeam>,
        Write<'a, GameInfo>,
        Write<'a, StarManager>,
        Write<'a, AttackManager>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut pos_storage,
             mut shot_storage,
             mut enemy_storage,
             mut coll_rect_storage,
             mut seqanime_storage,
             mut drawable_storage,
             zako_storage,
             mut owl_storage,
             mut troops_storage,
             mut recaptured_fighter_storage,
             mut player_storage,
             mut tractor_beam_storage,
             mut game_info,
             mut star_manager,
             mut attack_manager) = data;

        let mut colls: Vec<(Entity, Entity)> = Vec::new();
        for (shot, shot_pos, shot_coll_rect, shot_entity) in (&mut shot_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
            let shot_collboxes = [
                Some(pos_to_coll_box(&shot_pos.0, &shot_coll_rect)),
                shot.dual.map(|dual| pos_to_coll_box(&pos_storage.get(dual).unwrap().0, &shot_coll_rect)),
            ];
            let mut hit = false;
            for shot_collbox in shot_collboxes.iter().flat_map(|x| x) {
                for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in (&enemy_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
                    let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                    if shot_collbox.check_collision(&enemy_collbox) {
                        colls.push((enemy_entity, shot.player_entity));
                        hit = true;
                        break;
                    }
                }
            }
            if hit {
                delete_myshot(shot, shot_entity, &entities);
            }
        }

        for (enemy_entity, player_entity) in colls.iter() {
            set_enemy_damage(
                *enemy_entity, 1, &entities, &mut enemy_storage, &mut pos_storage, &zako_storage, &mut owl_storage,
                &mut troops_storage, &mut coll_rect_storage, &mut seqanime_storage, &mut drawable_storage,
                &mut recaptured_fighter_storage, &mut player_storage, &mut tractor_beam_storage,
                &mut attack_manager, &mut star_manager, &mut game_info, *player_entity);
        }
    }
}

pub struct SysCollCheckPlayerEnemy;
impl<'a> System<'a> for SysCollCheckPlayerEnemy {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SequentialSpriteAnime>,
        WriteStorage<'a, SpriteDrawable>,
        ReadStorage<'a, Zako>,
        WriteStorage<'a, Owl>,
        WriteStorage<'a, Troops>,
        WriteStorage<'a, RecapturedFighter>,
        WriteStorage<'a, TractorBeam>,
        Write<'a, GameInfo>,
        Write<'a, StarManager>,
        Write<'a, AttackManager>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut pos_storage,
             mut player_storage,
             mut enemy_storage,
             mut coll_rect_storage,
             mut seqanime_storage,
             mut drawable_storage,
             zako_storage,
             mut owl_storage,
             mut troops_storage,
             mut recaptured_fighter_storage,
             mut tractor_beam_storage,
             mut game_info,
             mut star_manager,
             mut attack_manager) = data;

        let mut colls: Vec<(Entity, Vec2I, bool, Entity)> = Vec::new();
        for (player, player_pos, player_coll_rect, player_entity) in (&player_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
            let player_poses = [
                Some(player_pos.0.clone()),
                player.dual.map(|dual| pos_storage.get(dual).unwrap().0.clone()),
            ];
            for (i, pl_pos) in player_poses.iter().flat_map(|x| x).enumerate() {
                let player_collbox = pos_to_coll_box(&pl_pos, player_coll_rect);
                for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in (&enemy_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
                    let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                    if player_collbox.check_collision(&enemy_collbox) {
                        colls.push((player_entity, *pl_pos, i != 0, enemy_entity));
                        break;
                    }
                }
            }
        }

        for (player_entity, player_pos, dual, enemy_entity) in colls.iter() {
            set_enemy_damage(
                *enemy_entity, 1, &entities, &mut enemy_storage, &mut pos_storage, &zako_storage, &mut owl_storage,
                &mut troops_storage, &mut coll_rect_storage, &mut seqanime_storage, &mut drawable_storage,
                &mut recaptured_fighter_storage, &mut player_storage, &mut tractor_beam_storage,
                &mut attack_manager, &mut star_manager, &mut game_info, *player_entity);

            create_player_explosion_effect(player_pos, &entities, &mut pos_storage, &mut seqanime_storage, &mut drawable_storage);

            let player = player_storage.get_mut(*player_entity).unwrap();
            set_damage_to_player(
                player, *dual, *player_entity, &mut pos_storage, &mut coll_rect_storage, &mut drawable_storage, &entities,
                &mut game_info, &mut star_manager, &mut attack_manager);
        }
    }
}

fn set_damage_to_player<'a>(
    player: &mut Player, dual: bool, entity: Entity, pos_storage: &mut WriteStorage<'a, Posture>, coll_rect_storage: &mut WriteStorage<'a, CollRect>, drawable_storage: &mut WriteStorage<'a, SpriteDrawable>, entities: &Entities<'a>,
    game_info: &mut GameInfo, star_manager: &mut StarManager, attack_manager: &mut AttackManager,
) {
    if crash_player(player, dual, entity, pos_storage, drawable_storage, coll_rect_storage, entities) {
        if game_info.capture_state != CaptureState::Recapturing {
            star_manager.set_stop(true);
        }
        game_info.crash_player(true, attack_manager);
    } else {
        game_info.crash_player(false, attack_manager);
    }
}

pub struct SysCollCheckPlayerEneShot;
impl<'a> System<'a> for SysCollCheckPlayerEneShot {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, EneShot>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SequentialSpriteAnime>,
        WriteStorage<'a, SpriteDrawable>,
        Write<'a, GameInfo>,
        Write<'a, StarManager>,
        Write<'a, AttackManager>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             mut pos_storage,
             mut player_storage,
             eneshot_storage,
             mut coll_rect_storage,
             mut seqanime_storage,
             mut drawable_storage,
             mut game_info,
             mut star_manager,
             mut attack_manager) = data;

        let mut colls: Vec<(Entity, Vec2I, bool)> = Vec::new();
        for (player, player_pos, player_coll_rect, player_entity) in (&player_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
            let player_collboxes = [
                Some(pos_to_coll_box(&player_pos.0, &player_coll_rect)),
                player.dual.map(|dual| pos_to_coll_box(&pos_storage.get(dual).unwrap().0, &player_coll_rect)),
            ];
            for (i, player_collbox) in player_collboxes.iter().flat_map(|x| x).enumerate() {
                for (_eneshot, eneshot_pos, eneshot_coll_rect, eneshot_entity) in (&eneshot_storage, &pos_storage, &coll_rect_storage, &*entities).join() {
                    let enemy_collbox = CollBox { top_left: &round_vec(&eneshot_pos.0) + &eneshot_coll_rect.offset, size: eneshot_coll_rect.size };
                    if player_collbox.check_collision(&enemy_collbox) {
                        colls.push((player_entity, player_pos.0.clone(), i != 0));
                        entities.delete(eneshot_entity).unwrap();
                        break;
                    }
                }
            }
        }

        for (player_entity, player_pos, dual) in colls.iter() {
            create_player_explosion_effect(player_pos, &entities, &mut pos_storage, &mut seqanime_storage, &mut drawable_storage);

            let player = player_storage.get_mut(*player_entity).unwrap();
            set_damage_to_player(
                player, *dual, *player_entity, &mut pos_storage, &mut coll_rect_storage, &mut drawable_storage, &entities,
                &mut game_info, &mut star_manager, &mut attack_manager);
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

pub struct SysRecaptureFighter;
impl<'a> System<'a> for SysRecaptureFighter {
    type SystemData = (
        WriteStorage<'a, RecapturedFighter>,
        WriteStorage<'a, Posture>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, CollRect>,
        WriteStorage<'a, SpriteDrawable>,
        Entities<'a>,
        Write<'a, AttackManager>,
        Write<'a, GameInfo>,
    );

    fn run(&mut self, (mut recaptured_fighter_storage, mut pos_storage, mut player_storage, mut coll_rect_storage, mut drawable_storage, entities, mut attack_manager, mut game_info): Self::SystemData) {
        for (recaptured_fighter, entity) in (&mut recaptured_fighter_storage, &*entities).join() {
            recapture_fighter(recaptured_fighter, entity, &entities, &mut player_storage, &mut pos_storage, &mut coll_rect_storage, &mut drawable_storage, &mut attack_manager, &mut game_info);
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
    type SystemData = (Read<'a, StarManager>, ReadStorage<'a, Posture>, ReadStorage<'a, SpriteDrawable>, ReadStorage<'a, SpriteColor>, Read<'a, GameInfo>, Read<'a, StageIndicator>);

    fn run(&mut self, (star_manager, pos_storage, drawable_storage, tex_color_storage, game_info, stage_indicator): Self::SystemData) {
        let renderer = &mut self.0;

        star_manager.draw(*renderer);
        for (posture, drawable, tex_color_opt) in (&pos_storage, &drawable_storage, (&tex_color_storage).maybe()).join() {
            let pos = &round_vec(&posture.0) + &drawable.offset;
            let angle = quantize_angle(posture.1, ANGLE_DIV);

            let tex_color = tex_color_opt.unwrap_or_else(|| &SpriteColor(255, 255, 255));
            renderer.set_sprite_texture_color_mod(drawable.sprite_name, tex_color.0, tex_color.1, tex_color.2);

            if angle == 0 {
                renderer.draw_sprite(drawable.sprite_name, &pos);
            } else {
                renderer.draw_sprite_rot(drawable.sprite_name, &pos, angle, None);
            }
        }

        stage_indicator.draw(*renderer);

        if game_info.left_ship > 0 {
            let disp_count = std::cmp::min(game_info.left_ship - 1, 8);
            renderer.set_sprite_texture_color_mod("rustacean", 255, 255, 255);
            for i in 0..disp_count {
                renderer.draw_sprite("rustacean", &Vec2I::new(i as i32 * 16, HEIGHT - 16));
            }
        }

        game_info.score_holder.draw(*renderer, (game_info.frame_count & 31) < 16);

        match game_info.game_state {
            GameState::StartStage => {
                renderer.set_texture_color_mod("font", 0, 255, 255);
                renderer.draw_str("font", 10 * 8, 18 * 8, &format!("STAGE {}", game_info.stage + 1));
            }
            GameState::WaitReady | GameState::WaitReady2 => {
                if game_info.left_ship > 1 || game_info.game_state == GameState::WaitReady2 {
                    renderer.set_texture_color_mod("font", 0, 255, 255);
                    renderer.draw_str("font", (28 - 6) / 2 * 8, 18 * 8, "READY");
                }
            }
            GameState::Captured => {
                if game_info.count < 120 {
                    renderer.set_texture_color_mod("font", 255, 0, 0);
                    renderer.draw_str("font", (28 - 16) / 2 * 8, 19 * 8, "FIGHTER CAPTURED");
                }
            }
            GameState::GameOver => {
                renderer.set_texture_color_mod("font", 0, 255, 255);
                renderer.draw_str("font", (28 - 8) / 2 * 8, 18 * 8, "GAME OVER");
            }
            _ => {}
        }
    }
}

//

fn pos_to_coll_box(pos: &Vec2I, coll_rect: &CollRect) -> CollBox {
    CollBox { top_left: &round_vec(pos) + &coll_rect.offset, size: coll_rect.size }
}

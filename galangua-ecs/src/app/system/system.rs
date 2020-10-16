use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

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
use galangua_common::framework::types::{Vec2I, ZERO_VEC};
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{quantize_angle, round_vec};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;
use crate::app::resources::*;

use super::system_effect::*;
use super::system_enemy::*;
use super::system_owl::*;
use super::system_player::*;

#[system]
#[write_component(Player)]
#[write_component(Posture)]
pub fn update_game_controller(
    world: &mut SubWorld,
    #[resource] game_info: &mut GameInfo,
    #[resource] stage_indicator: &mut StageIndicator,
    #[resource] formation: &mut Formation,
    #[resource] appearance_manager: &mut AppearanceManager,
    #[resource] attack_manager: &mut AttackManager,
    #[resource] star_manager: &mut StarManager,
    commands: &mut CommandBuffer,
) {
    game_info.update(
        stage_indicator, formation, appearance_manager, attack_manager,
        star_manager, world, commands);
}

#[system(for_each)]
#[write_component(Posture)]
pub fn move_player(
    player: &mut Player, entity: &Entity,
    #[resource] pad: &Pad,
    #[resource] game_info: &mut GameInfo,
    #[resource] attack_manager: &mut AttackManager,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) {
    do_move_player(player, pad, *entity, game_info, attack_manager, world, commands);
}

#[system(for_each)]
#[read_component(MyShot)]
pub fn fire_myshot(player: &Player, posture: &Posture, entity: &Entity, world: &mut SubWorld, #[resource] pad: &Pad, commands: &mut CommandBuffer) {
    let shot_count = <&MyShot>::query().iter(world).count();
    if pad.is_trigger(PadBit::A) && shot_count < 2 {
        do_fire_myshot(player, posture, *entity, commands);
    }
}

#[system(for_each)]
#[write_component(Posture)]
pub fn move_myshot(shot: &MyShot, entity: &Entity, world: &mut SubWorld, commands: &mut CommandBuffer) {
    do_move_myshot(shot, *entity, world, commands);
}

#[system]
pub fn move_formation(#[resource] formation: &mut Formation) {
    formation.update();
}

#[system]
#[read_component(Zako)]
pub fn run_appearance_manager(world: &mut SubWorld, #[resource] appearance_manager: &mut AppearanceManager, #[resource] attack_manager: &mut AttackManager, #[resource] formation: &mut Formation, #[resource] game_info: &mut GameInfo, commands: &mut CommandBuffer) {
    if appearance_manager.done {
        return;
    }

    let accessor = SysAppearanceManagerAccessor(world);
    let new_borns_opt = appearance_manager.update(&accessor);
    if let Some(new_borns) = new_borns_opt {
        new_borns.into_iter().for_each(|e| {
            let sprite_name = match e.enemy_type {
                EnemyType::Bee => "gopher1",
                EnemyType::Butterfly => "dman1",
                EnemyType::Owl => "cpp11",
                EnemyType::CapturedFighter => "rustacean_captured",
            };

            let enemy = Enemy { enemy_type: e.enemy_type, formation_index: e.fi, is_formation: false };
            let posture = Posture(e.pos, 0);
            let speed = Speed(0, 0);
            let coll_rect = CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) };
            let drawable = SpriteDrawable {sprite_name, offset: Vec2I::new(-8, -8)};
            if e.enemy_type != EnemyType::Owl {
                let zako = Zako { state: ZakoState::Appearance, traj: Some(e.traj), target_pos: ZERO_VEC };
                commands.push((enemy, zako, posture, speed, coll_rect, drawable));
            } else {
                let owl = create_owl(e.traj);
                commands.push((enemy, owl, posture, speed, coll_rect, drawable));
            }
            game_info.alive_enemy_count += 1;
        });
    }

    if appearance_manager.done {
        formation.done_appearance();
        attack_manager.set_enable(true);
    }
}

struct SysAppearanceManagerAccessor<'a, 'b>(&'a mut SubWorld<'b>);
impl<'a, 'b> AppearanceManagerAccessor for SysAppearanceManagerAccessor<'a, 'b> {
    fn is_stationary(&self) -> bool {
        <&Zako>::query().iter(self.0)
            .all(|x| x.state == ZakoState::Formation)
    }
}

#[system]
#[write_component(Zako)]
#[write_component(Owl)]
#[write_component(Enemy)]
#[write_component(Posture)]
#[write_component(Speed)]
#[read_component(Player)]
pub fn run_attack_manager(world: &mut SubWorld, #[resource] attack_manager: &mut AttackManager, #[resource] game_info: &mut GameInfo, commands: &mut CommandBuffer) {
    let result = {
        let accessor = SysAttackManagerAccessor(world, game_info);
        attack_manager.update(&accessor)
    };
    if let Some((fi, capture_attack)) = result {
        let get_player_pos = || {
            for (_player, posture) in <(&Player, &Posture)>::query().iter(world) {
                return Some(posture.0.clone());
            }
            None
        };

        let entity_opt = <(&Enemy, Option<&Owl>, Entity)>::query().iter(world)
            .find_map(|(enemy, owl, entity)| {
                if enemy.formation_index == fi {
                    Some((*entity, owl.is_some()))
                } else {
                    None
                }
            });
        if let Some((entity, is_owl)) = entity_opt {
            if is_owl {
                let player_pos = get_player_pos().unwrap();
                let (mut subworld1, mut subworld2) = world.split::<(&mut Owl, &mut Speed)>();
                let (owl, speed) = <(&mut Owl, &mut Speed)>::query().get_mut(&mut subworld1, entity).unwrap();
                owl_start_attack(owl, capture_attack, speed, &player_pos, entity, &mut subworld2, commands);
                if capture_attack {
                    game_info.capture_state = CaptureState::CaptureAttacking;
                    game_info.capture_enemy_fi = fi;
                }
            } else {
                let (zako, enemy, posture) = <(&mut Zako, &mut Enemy, &mut Posture)>::query().get_mut(world, entity).unwrap();
                zako_start_attack(zako, enemy, posture);
            }
            attack_manager.put_attacker(&fi);
        }
    }
}

struct SysAttackManagerAccessor<'a, 'b>(&'a mut SubWorld<'b>, &'a GameInfo);
impl<'a, 'b> SysAttackManagerAccessor<'a, 'b> {
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Enemy> {
        <&Enemy>::query().iter(self.0)
            .find(|enemy| enemy.formation_index == *formation_index)
    }
}
impl<'a, 'b> AttackManagerAccessor for SysAttackManagerAccessor<'a, 'b> {
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

#[system(for_each)]
#[read_component(Player)]
#[write_component(Posture)]
pub fn move_zako(
    enemy: &mut Enemy, zako: &mut Zako, speed: &mut Speed, entity: &Entity,
    world: &mut SubWorld,
    #[resource] formation: &Formation,
    #[resource] game_info: &mut GameInfo,
    commands: &mut CommandBuffer,
) {
    do_move_zako(zako, *entity, enemy, speed, formation, game_info, world, commands);
}

#[system(for_each)]
#[write_component(TractorBeam)]
#[write_component(Enemy)]
#[write_component(Zako)]
#[write_component(Troops)]
pub fn move_owl(owl: &mut Owl, posture: &mut Posture, speed: &mut Speed, entity: &Entity, world: &mut SubWorld, #[resource] formation: &Formation, #[resource] game_info: &mut GameInfo, commands: &mut CommandBuffer) {
    do_move_owl(owl, *entity, posture, speed, formation, game_info, world, commands);
}

#[system(for_each)]
#[write_component(Posture)]
pub fn move_troops(troops: &mut Troops, owl: &mut Owl, entity: &Entity, world: &mut SubWorld) {
    update_troops(troops, *entity, owl, world);
}

#[system(for_each)]
#[write_component(Player)]
#[write_component(Posture)]
pub fn move_tractor_beam(
    tractor_beam: &mut TractorBeam, owl: &mut Owl, enemy: &Enemy, entity: &Entity,
    #[resource] game_info: &mut GameInfo,
    #[resource] star_manager: &mut StarManager,
    #[resource] attack_manager: &mut AttackManager,
    world: &mut SubWorld, commands: &mut CommandBuffer,
) {
    do_move_tractor_beam(tractor_beam, *entity, owl, enemy, game_info, star_manager, attack_manager, world, commands);
}

#[system]
#[read_component(MyShot)]
#[read_component(Posture)]
#[read_component(CollRect)]
#[read_component(Zako)]
#[write_component(Enemy)]
#[write_component(Owl)]
#[write_component(TractorBeam)]
#[write_component(Troops)]
#[write_component(SpriteDrawable)]
#[write_component(Player)]
pub fn coll_check_myshot_enemy(
    world: &mut SubWorld,
    #[resource] star_manager: &mut StarManager,
    #[resource] attack_manager: &mut AttackManager,
    #[resource] game_info: &mut GameInfo,
    commands: &mut CommandBuffer,
) {
    let mut colls: Vec<(Entity, Entity)> = Vec::new();
    for (shot, shot_pos, shot_coll_rect, shot_entity) in <(&MyShot, &Posture, &CollRect, Entity)>::query().iter(world) {
        let shot_collboxes = [
            Some(pos_to_coll_box(&shot_pos.0, &shot_coll_rect)),
            shot.dual.map(|dual| pos_to_coll_box(&<&Posture>::query().get(world, dual).unwrap().0, &shot_coll_rect)),
        ];
        let mut hit = false;
        for shot_collbox in shot_collboxes.iter().flat_map(|x| x) {
            for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in <(&Enemy, &Posture, &CollRect, Entity)>::query().iter(world) {
                let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                if shot_collbox.check_collision(&enemy_collbox) {
                    colls.push((*enemy_entity, shot.player_entity));
                    hit = true;
                    break;
                }
            }
            if hit {
                delete_myshot(shot, *shot_entity, commands);
            }
        }
    }

    for (enemy_entity, player_entity) in colls {
        let enemy_type = <&Enemy>::query().get(world, enemy_entity).unwrap().enemy_type;
        set_enemy_damage(enemy_type, enemy_entity, 1, player_entity, star_manager, attack_manager, game_info, world, commands);
    }
}

#[system]
#[read_component(CollRect)]
#[read_component(Zako)]
#[write_component(Posture)]
#[write_component(Player)]
#[write_component(Enemy)]
#[write_component(Owl)]
#[write_component(TractorBeam)]
#[write_component(Troops)]
#[write_component(SpriteDrawable)]
#[write_component(Player)]
pub fn coll_check_player_enemy(
    world: &mut SubWorld,
    #[resource] star_manager: &mut StarManager,
    #[resource] attack_manager: &mut AttackManager,
    #[resource] game_info: &mut GameInfo,
    commands: &mut CommandBuffer,
) {
    let mut colls: Vec<(Entity, Vec2I, bool, Entity)> = Vec::new();
    for (player, player_pos, player_coll_rect, player_entity) in <(&Player, &Posture, &CollRect, Entity)>::query().iter(world) {
        let player_poses = [
            Some(player_pos.0.clone()),
            player.dual.map(|dual| <&Posture>::query().get(world, dual).unwrap().0.clone()),
        ];
        for (i, pl_pos) in player_poses.iter().flat_map(|x| x).enumerate() {
            let player_collbox = pos_to_coll_box(&pl_pos, player_coll_rect);
            for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in <(&Enemy, &Posture, &CollRect, Entity)>::query().iter(world) {
                let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
                if player_collbox.check_collision(&enemy_collbox) {
                    colls.push((*player_entity, *pl_pos, i != 0, *enemy_entity));
                    break;
                }
            }
        }
    }

    for (player_entity, pl_pos, dual, enemy_entity) in colls {
        let enemy_type = <&Enemy>::query().get(world, enemy_entity).unwrap().enemy_type;
        set_enemy_damage(enemy_type, enemy_entity, 100, player_entity, star_manager, attack_manager, game_info, world, commands);

        create_player_explosion_effect(&pl_pos, commands);

        let (mut subworld1, mut subworld2) = world.split::<&mut Player>();
        let player = <&mut Player>::query().get_mut(&mut subworld1, player_entity).unwrap();
        if crash_player(player, dual, player_entity, &mut subworld2, commands) {
            star_manager.set_stop(true);
            game_info.crash_player(true, attack_manager);
            star_manager.set_stop(true);
        } else {
            game_info.crash_player(false, attack_manager);
        }
    }
}

#[system(for_each)]
pub fn move_sequential_anime(anime: &mut SequentialSpriteAnime, drawable: &mut SpriteDrawable, entity: &Entity, commands: &mut CommandBuffer) {
    if let Some(sprite_name) = update_seqanime(anime) {
        drawable.sprite_name = sprite_name;
    } else {
        commands.remove(*entity);
    }
}

#[system]
#[write_component(RecapturedFighter)]
#[write_component(Player)]
#[write_component(Posture)]
pub fn recapture_fighter(world: &mut SubWorld, #[resource] attack_manager: &mut AttackManager, #[resource] game_info: &mut GameInfo, commands: &mut CommandBuffer) {
    let (mut subworld1, mut subworld2) = world.split::<&mut RecapturedFighter>();
    for (recaptured_fighter, entity) in <(&mut RecapturedFighter, Entity)>::query().iter_mut(&mut subworld1) {
        do_recapture_fighter(recaptured_fighter, *entity, attack_manager, game_info, &mut subworld2, commands);
    }
}

#[system]
pub fn move_star(#[resource] star_manager: &mut StarManager) {
    star_manager.update();
}

pub fn draw_system<R: RendererTrait>(world: &World, resources: &Resources, renderer: &mut R) {
    let star_manager = resources.get::<StarManager>().unwrap();
    star_manager.draw(renderer);

    for (posture, drawable) in <(&Posture, &SpriteDrawable)>::query().iter(world) {
        let pos = &round_vec(&posture.0) + &drawable.offset;
        let angle = quantize_angle(posture.1, ANGLE_DIV);
        if angle == 0 {
            renderer.draw_sprite(drawable.sprite_name, &pos);
        } else {
            renderer.draw_sprite_rot(drawable.sprite_name, &pos, angle, None);
        }
    }

    let stage_indicator = resources.get::<StageIndicator>().unwrap();
    stage_indicator.draw(renderer);

    let game_info = resources.get::<GameInfo>().unwrap();
    if game_info.left_ship > 0 {
        let disp_count = std::cmp::min(game_info.left_ship - 1, 8);
        for i in 0..disp_count {
            renderer.draw_sprite("rustacean", &Vec2I::new(i as i32 * 16, HEIGHT - 16));
        }
    }

    game_info.score_holder.draw(renderer, /*(self.frame_count & 31) < 16*/ true);

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

//

fn pos_to_coll_box(pos: &Vec2I, coll_rect: &CollRect) -> CollBox {
    CollBox { top_left: &round_vec(pos) + &coll_rect.offset, size: coll_rect.size }
}

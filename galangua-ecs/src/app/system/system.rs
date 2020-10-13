use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::appearance_manager::AppearanceManager;
use galangua_common::app::game::appearance_manager::Accessor as AppearanceManagerAccessor;
use galangua_common::app::game::attack_manager::AttackManager;
use galangua_common::app::game::attack_manager::Accessor as AttackManagerAccessor;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::{CaptureState, EnemyType, FormationIndex};
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{quantize_angle, round_vec, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;
use crate::app::resources::*;

use super::system_effect::*;
use super::system_enemy::*;
use super::system_player::*;

#[system]
pub fn update_pad(#[resource] pad: &mut Pad) {
    pad.update();
}

#[system(for_each)]
pub fn move_player(player: &mut Player, posture: &mut Posture, entity: &Entity, #[resource] pad: &Pad, commands: &mut CommandBuffer) {
    do_move_player(player, *entity, pad, posture, commands);
}

#[system(for_each)]
#[read_component(MyShot)]
pub fn fire_myshot(player: &Player, posture: &Posture, world: &mut SubWorld, #[resource] pad: &Pad, commands: &mut CommandBuffer) {
    let shot_count = <&MyShot>::query().iter(world).count();
    if can_player_fire(player) && pad.is_trigger(PadBit::A) {
        if shot_count < 2 {
            commands.push((
                MyShot,
                posture.clone(),
                CollRect { offset: Vec2I::new(-1, -4), size: Vec2I::new(1, 8) },
                SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)},
            ));
        }
    }
}

#[system(for_each)]
pub fn move_myshot(_myshot: &MyShot, pos: &mut Posture, entity: &Entity, commands: &mut CommandBuffer) {
    let mut pos = &mut pos.0;
    pos.y -= MYSHOT_SPEED;
    if pos.y < 0 * ONE {
        commands.remove(*entity);

    }
}

#[system]
pub fn move_formation(#[resource] formation: &mut Formation) {
    formation.update();
}

#[system]
#[read_component(Zako)]
pub fn run_appearance_manager(world: &mut SubWorld, #[resource] appearance_manager: &mut AppearanceManager, #[resource] attack_manager: &mut AttackManager, #[resource] formation: &mut Formation, commands: &mut CommandBuffer) {
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
                let zako = Zako { state: ZakoState::Appearance, traj: Some(e.traj) };
                commands.push((enemy, zako, posture, speed, coll_rect, drawable));
            } else {
                let owl = create_owl(e.traj);
                commands.push((enemy, owl, posture, speed, coll_rect, drawable));
            }
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
pub fn run_attack_manager(world: &mut SubWorld, #[resource] attack_manager: &mut AttackManager, #[resource] game_info: &mut GameInfo) {
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
        let player_pos = get_player_pos().unwrap();

        <(&mut Enemy, Option<&mut Zako>, Option<&mut Owl>, &mut Posture, &mut Speed)>::query().iter_mut(world)
            .filter(|(enemy, ..)| enemy.formation_index == fi)
            .for_each(|(enemy, zako_opt, owl_opt, posture, speed)| {
                if let Some(zako) = zako_opt {
                    zako_start_attack(zako, enemy, &posture);
                }
                if let Some(owl) = owl_opt {
                    owl_start_attack(owl, enemy, posture, capture_attack, speed, &player_pos);
                    if capture_attack {
                        game_info.capture_state = CaptureState::CaptureAttacking;
                        game_info.capture_enemy_fi = fi;
                    }
                }
                attack_manager.put_attacker(&fi);
            });
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
pub fn move_zako(enemy: &mut Enemy, zako: &mut Zako, posture: &mut Posture, speed: &mut Speed, #[resource] formation: &Formation) {
    do_move_zako(zako, enemy, posture, speed, formation);
}

#[system(for_each)]
pub fn move_owl(enemy: &mut Enemy, entity: &Entity, owl: &mut Owl, posture: &mut Posture, tractor_beam: Option<&mut TractorBeam>, speed: &mut Speed, #[resource] formation: &Formation, #[resource] game_info: &mut GameInfo, commands: &mut CommandBuffer) {
    do_move_owl(owl, enemy, *entity, posture, speed, tractor_beam, formation, game_info, commands);
}

#[system(for_each)]
#[write_component(Posture)]
pub fn move_troops(troops: &mut Troops, owl: &mut Owl, entity: &Entity, world: &mut SubWorld) {
    update_troops(troops, *entity, owl, world);
}

#[system(for_each)]
#[write_component(Player)]
#[write_component(Posture)]
pub fn move_tractor_beam(tractor_beam: &mut TractorBeam, owl: &mut Owl, enemy: &Enemy, entity: &Entity, world: &mut SubWorld, #[resource] game_info: &mut GameInfo, commands: &mut CommandBuffer) {
    do_move_tractor_beam(tractor_beam, *entity, owl, enemy, game_info, world, commands);
}

#[system]
#[read_component(MyShot)]
#[read_component(Enemy)]
#[read_component(Posture)]
#[read_component(CollRect)]
pub fn coll_check_myshot_enemy(world: &mut SubWorld, commands: &mut CommandBuffer) {
    for (_shot, shot_pos, shot_coll_rect, shot_entity) in <(&MyShot, &Posture, &CollRect, Entity)>::query().iter(world) {
        let shot_collbox = CollBox { top_left: &round_vec(&shot_pos.0) + &shot_coll_rect.offset, size: shot_coll_rect.size };
        for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in <(&Enemy, &Posture, &CollRect, Entity)>::query().iter(world) {
            let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
            if shot_collbox.check_collision(&enemy_collbox) {
                let pos = enemy_pos.0.clone();
                commands.remove(*enemy_entity);
                commands.remove(*shot_entity);
                create_enemy_explosion_effect(&pos, commands);
                break;
            }
        }
    }
}

#[system]
#[write_component(Player)]
#[read_component(Enemy)]
#[read_component(Posture)]
#[read_component(CollRect)]
pub fn coll_check_player_enemy(world: &mut SubWorld, commands: &mut CommandBuffer) {
    let mut colls: Vec<Entity> = Vec::new();
    for (_player, player_pos, player_coll_rect, player_entity) in <(&Player, &Posture, &CollRect, Entity)>::query().iter(world) {
        let player_collbox = CollBox { top_left: &round_vec(&player_pos.0) + &player_coll_rect.offset, size: player_coll_rect.size };
        for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in <(&Enemy, &Posture, &CollRect, Entity)>::query().iter(world) {
            let enemy_collbox = CollBox { top_left: &round_vec(&enemy_pos.0) + &enemy_coll_rect.offset, size: enemy_coll_rect.size };
            if player_collbox.check_collision(&enemy_collbox) {
                let pl_pos = player_pos.0.clone();
                let ene_pos = enemy_pos.0.clone();
                commands.remove(*enemy_entity);
                create_player_explosion_effect(&pl_pos, commands);
                create_enemy_explosion_effect(&ene_pos, commands);
                colls.push(*player_entity);
                break;
            }
        }
    }

    for player_entity in colls {
        let player = <&mut Player>::query().get_mut(world, player_entity).unwrap();
        crash_player(player, player_entity, commands);
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
}

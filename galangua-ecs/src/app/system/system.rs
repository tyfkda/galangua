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
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{quantize_angle, round_vec, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use crate::app::components::*;

use super::system_enemy::*;

#[system]
pub fn update_pad(#[resource] pad: &mut Pad) {
    pad.update();
}

#[system(for_each)]
pub fn move_player(_player: &Player, posture: &mut Posture, #[resource] pad: &Pad) {
    let mut pos = &mut posture.0;
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
#[read_component(MyShot)]
pub fn fire_myshot(_player: &Player, posture: &Posture, world: &mut SubWorld, #[resource] pad: &Pad, commands: &mut CommandBuffer) {
    let shot_count = <&MyShot>::query().iter(world).count();
    if pad.is_trigger(PadBit::A) {
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
        let tuples = new_borns.into_iter().map(|e| {
            let sprite_name = match e.enemy_type {
                EnemyType::Bee => "gopher1",
                EnemyType::Butterfly => "dman1",
                EnemyType::Owl => "cpp11",
                EnemyType::CapturedFighter => "rustacean_captured",
            };
            (
                Enemy { enemy_type: e.enemy_type, formation_index: e.fi },
                Zako { state: ZakoState::Appearance, traj: Some(e.traj) },
                Posture(e.pos, 0),
                Speed(0, 0),
                CollRect { offset: Vec2I::new(-6, -6), size: Vec2I::new(12, 12) },
                SpriteDrawable {sprite_name, offset: Vec2I::new(-8, -8)},
            )
        });
        commands.extend(tuples);
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
#[read_component(Enemy)]
#[read_component(Posture)]
pub fn run_attack_manager(world: &mut SubWorld, #[resource] attack_manager: &mut AttackManager) {
    let result = {
        let accessor = SysAttackManagerAccessor(world);
        attack_manager.update(&accessor)
    };
    if let Some((fi, capture_attack)) = result {
        <(&Enemy, &mut Zako, &Posture)>::query().iter_mut(world)
            .filter(|(enemy, ..)| enemy.formation_index == fi)
            .for_each(|(enemy, zako, posture)| {
                zako_start_attack(zako, &enemy, &posture, capture_attack);
                attack_manager.put_attacker(&fi);
            });
    }
}

struct SysAttackManagerAccessor<'a, 'b>(&'a mut SubWorld<'b>);
impl<'a, 'b> SysAttackManagerAccessor<'a, 'b> {
    fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Enemy> {
        <&Enemy>::query().iter(self.0)
            .find(|enemy| enemy.formation_index == *formation_index)
    }
    fn get_zako_at(&self, formation_index: &FormationIndex) -> Option<&Zako> {
        <(&Enemy, &Zako)>::query().iter(self.0)
            .find(|(enemy, _zako)| enemy.formation_index == *formation_index)
            .map(|(_enemy, zako)| zako)
    }
}
impl<'a, 'b> AttackManagerAccessor for SysAttackManagerAccessor<'a, 'b> {
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

#[system(for_each)]
pub fn move_zako(enemy: &Enemy, zako: &mut Zako, posture: &mut Posture, speed: &mut Speed, #[resource] formation: &Formation) {
    do_move_zako(zako, enemy, posture, speed, &formation);
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
                commands.remove(*enemy_entity);
                commands.remove(*shot_entity);
                break;
            }
        }
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

use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{round_vec, ONE};
use galangua_common::util::pad::{Pad, PadBit};

use super::components::*;

#[system]
pub fn update_pad(#[resource] pad: &mut Pad) {
    pad.update();
}

#[system(for_each)]
pub fn move_player(_player: &Player, pos: &mut Pos, #[resource] pad: &Pad) {
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

#[system(for_each)]
#[read_component(MyShot)]
pub fn fire_myshot(_player: &Player, pos: &Pos, world: &mut SubWorld, #[resource] pad: &Pad, commands: &mut CommandBuffer) {
    let shot_count = <&MyShot>::query().iter(world).count();
    if pad.is_trigger(PadBit::A) {
        if shot_count < 2 {
            commands.push((
                MyShot,
                Pos(pos.0.clone()),
                CollRect { offset: Vec2I::new(-1, -4), size: Vec2I::new(1, 8) },
                SpriteDrawable {sprite_name: "myshot", offset: Vec2I::new(-2, -4)},
            ));
        }
    }
}

#[system(for_each)]
pub fn move_myshot(_myshot: &MyShot, pos: &mut Pos, entity: &Entity, commands: &mut CommandBuffer) {
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

#[system(for_each)]
pub fn move_enemy(enemy: &Enemy, pos: &mut Pos, #[resource] formation: &Formation) {
    *pos = Pos(formation.pos(&enemy.formation_index));
}

#[system]
#[read_component(MyShot)]
#[read_component(Enemy)]
#[read_component(Pos)]
#[read_component(CollRect)]
pub fn coll_check_myshot_enemy(world: &mut SubWorld, commands: &mut CommandBuffer) {
    for (_shot, shot_pos, shot_coll_rect, shot_entity) in <(&MyShot, &Pos, &CollRect, Entity)>::query().iter(world) {
        let shot_collbox = CollBox { top_left: &round_vec(&shot_pos.0) + &shot_coll_rect.offset, size: shot_coll_rect.size };
        for (_enemy, enemy_pos, enemy_coll_rect, enemy_entity) in <(&Enemy, &Pos, &CollRect, Entity)>::query().iter(world) {
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

    for (pos, drawable) in <(&Pos, &SpriteDrawable)>::query().iter(world) {
        let pos = round_vec(&pos.0);
        renderer.draw_sprite(drawable.sprite_name, &(&pos + &drawable.offset));
    }
}

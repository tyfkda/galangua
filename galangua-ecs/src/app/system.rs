use legion::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;

use galangua_common::app::consts::*;
use galangua_common::app::game::formation::Formation;
use galangua_common::app::game::star_manager::StarManager;
use galangua_common::app::game::traj::Accessor as TrajAccessor;
use galangua_common::app::game::FormationIndex;
use galangua_common::app::util::collision::CollBox;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{atan2_lut, calc_velocity, clamp, diff_angle, quantize_angle, round_vec, square, ANGLE, ONE, ONE_BIT};
use galangua_common::util::pad::{Pad, PadBit};

use super::components::*;

struct TrajAccessorImpl<'a> {
    formation: &'a Formation,
}
impl<'a> TrajAccessor for TrajAccessorImpl<'a> {
    fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }
    fn get_stage_no(&self) -> u16 { 0 }
}

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

#[system(for_each)]
pub fn move_zako(enemy: &Enemy, zako: &mut Zako, posture: &mut Posture, speed: &mut Speed, #[resource] formation: &Formation) {
    match zako.state {
        ZakoState::Formation => {
            posture.0 = formation.pos(&enemy.formation_index);
            let ang = ANGLE * ONE / 128;
            posture.1 -= clamp(posture.1, -ang, ang);
        }
        ZakoState::Attack => {
            if let Some(traj) = &mut zako.traj.as_mut() {
                let traj_accessor = TrajAccessorImpl { formation: &formation };
                let cont = traj.update(&traj_accessor);

                posture.0 = traj.pos();
                posture.1 = traj.angle;
                speed.0 = traj.speed;
                speed.1 = traj.vangle;
                //if let Some(wait) = traj.is_shot() {
                //    self.shot_wait = Some(wait);
                //}
                if !cont {
                    zako.traj = None;
                    zako.state = ZakoState::MoveToFormation;
                }
            }
        }
        ZakoState::MoveToFormation => {
            let target = formation.pos(&enemy.formation_index);
            let pos = &mut posture.0;
            let angle = &mut posture.1;
            let spd = &mut speed.0;
            let vangle = &mut speed.1;
            let diff = &target - &pos;
            let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
            let cont = if sq_distance > square(*spd >> (ONE_BIT / 2)) {
                let dlimit: i32 = *spd * 5 / 3;
                let target_angle = atan2_lut(-diff.y, diff.x);
                let d = diff_angle(target_angle, *angle);
                *angle += clamp(d, -dlimit, dlimit);
                *vangle = 0;
                *pos += &calc_velocity(*angle, *spd);
                true
            } else {
                *pos = target;
                *spd = 0;
                *vangle = 0;
                false
            };
            if !cont {
                zako.state = ZakoState::Formation;
            }
        }
    }
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

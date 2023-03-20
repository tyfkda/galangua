use super::enemy_base::{CoordinateTrait, FormationTrait};
use super::owl::{Owl, OwlState};
use super::zako::{Zako, ZakoState};
use super::{Accessor, DamageResult};

use galangua_common::app::consts::*;
use galangua_common::app::game::traj::Traj;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::util::collision::Collidable;
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{quantize_angle, round_vec};

#[cfg(debug_assertions)]
use galangua_common::app::game::traj_command::TrajCommand;

pub trait Enemy: Collidable + CoordinateTrait + FormationTrait {
    fn update(&mut self, accessor: &mut dyn Accessor) -> bool;
    fn draw(&self, renderer: &mut dyn RendererTrait, pat: usize);

    fn draw_sprite(&self, renderer: &mut dyn RendererTrait, sprite: &str, center: &Vec2I) {
        let angle = quantize_angle(self.angle(), ANGLE_DIV);
        let pos = round_vec(self.pos());
        renderer.draw_sprite_rot(sprite, &(&pos - center), angle, None);
    }

    fn is_formation(&self) -> bool;

    fn set_damage(&mut self, power: u32, accessor: &mut dyn Accessor) -> DamageResult;

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>);

    fn start_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor);
    fn set_to_troop(&mut self);
    fn set_to_formation(&mut self);

    #[cfg(debug_assertions)]
    fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool);
}

//================================================

pub fn create_enemy(
    enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
    fi: &FormationIndex,
) -> Box<dyn Enemy> {
    match enemy_type {
        EnemyType::Owl => Box::new(Owl::new(pos, angle, speed, fi)),
        _ => Box::new(Zako::new(enemy_type, pos, angle, speed, fi)),
    }
}

pub fn create_appearance_enemy(
    enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32,
    fi: &FormationIndex, traj: Traj,
) -> Box<dyn Enemy> {
    match enemy_type {
        EnemyType::Owl => {
            let mut owl = Owl::new(pos, angle, speed, fi);
            owl.base.traj = Some(traj);
            owl.set_state(OwlState::Appearance);
            Box::new(owl)
        }
        _ => {
            let mut zako = Zako::new(enemy_type, pos, angle, speed, fi);
            zako.base.traj = Some(traj);
            zako.set_state(ZakoState::Appearance);
            Box::new(zako)
        }
    }
}

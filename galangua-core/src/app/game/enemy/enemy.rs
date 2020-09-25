use super::owl::{Owl, OwlState};
use super::traj::Traj;
use super::zako::{Zako, ZakoState};
use super::{Accessor, DamageResult, EnemyType, FormationIndex};

use crate::app::util::collision::Collidable;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;

#[cfg(debug_assertions)]
use super::traj_command::TrajCommand;

pub trait Enemy: Collidable {
    fn update(&mut self, accessor: &mut dyn Accessor) -> bool;
    fn draw(&self, renderer: &mut dyn RendererTrait, pat: usize);

    fn pos(&self) -> &Vec2I;
    fn set_pos(&mut self, pos: &Vec2I);
    fn angle(&self) -> i32;

    fn is_formation(&self) -> bool;

    fn can_capture_attack(&self) -> bool;
    fn is_captured_fighter(&self) -> bool;
    fn formation_index(&self) -> &FormationIndex;

    fn set_damage(&mut self, power: u32, accessor: &mut dyn Accessor) -> DamageResult;

    fn update_troop(&mut self, add: &Vec2I, angle_opt: Option<i32>);

    fn set_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor);
    fn set_to_troop(&mut self);
    fn set_to_formation(&mut self);

    #[cfg(debug_assertions)]
    fn set_table_attack(&mut self, traj_command_vec: Vec<TrajCommand>, flip_x: bool);
}

////////////////////////////////////////////////

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

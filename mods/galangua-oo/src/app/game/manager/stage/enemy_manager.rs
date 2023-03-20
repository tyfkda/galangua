use array_macro::*;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use crate::app::game::enemy::ene_shot::EneShot;
use crate::app::game::enemy::enemy::{create_enemy, Enemy};
use crate::app::game::enemy::Accessor;
use crate::app::game::manager::EventType;

use galangua_common::app::consts::*;
use galangua_common::app::game::effect_table::to_earned_point_type;
use galangua_common::app::game::formation_table::X_COUNT;
use galangua_common::app::game::{EnemyType, FormationIndex};
use galangua_common::app::util::collision::{CollBox, Collidable};
use galangua_common::framework::types::Vec2I;
use galangua_common::framework::RendererTrait;
use galangua_common::util::math::{atan2_lut, calc_velocity, clamp, ANGLE, ONE};

const MAX_ENEMY_COUNT: usize = 70;

pub struct EnemyManager {
    enemies: [Option<Box<dyn Enemy>>; MAX_ENEMY_COUNT],
    pub(super) alive_enemy_count: u32,
    shots: [Option<EneShot>; MAX_ENE_SHOT_COUNT],
    shot_paused_count: u32,
    frame_count: u32,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: array![_ =>None; MAX_ENEMY_COUNT],
            alive_enemy_count: 0,
            shots: Default::default(),
            shot_paused_count: 0,
            frame_count: 0,
        }
    }

    pub fn start_next_stage(&mut self) {
        self.enemies = array![_ => None; MAX_ENEMY_COUNT];
        self.alive_enemy_count = 0;
        self.shots = Default::default();
        self.shot_paused_count = 0;
        self.frame_count = 0;
    }

    pub fn all_destroyed(&self) -> bool {
        self.shots.iter().all(|x| x.is_none())
    }

    pub fn update(&mut self, accessor: &mut impl Accessor) {
        self.frame_count = self.frame_count.wrapping_add(1);
        if self.shot_paused_count > 0 {
            self.shot_paused_count -= 1;
        }

        self.update_enemies(accessor);
        self.update_shots();
    }

    pub fn draw(&self, renderer: &mut impl RendererTrait) {
        let pat = ((self.frame_count >> 5) & 1) as usize;
        for enemy in self.enemies.iter().rev().flatten() {
            enemy.draw(renderer, pat);
        }
        for shot in self.shots.iter().flatten() {
            shot.draw(renderer);
        }
    }

    pub fn check_collision(
        &mut self, target: &CollBox, power: u32, accessor: &mut impl Accessor,
    ) -> bool {
        for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
            let enemy = enemy_opt.as_mut().unwrap();
            if let Some(collbox) = enemy.get_collbox() {
                if collbox.check_collision(target) {
                    let pos = *enemy.pos();
                    let result = enemy.set_damage(power, accessor);
                    if result.point > 0 {
                        accessor.push_event(EventType::AddScore(result.point));

                        if let Some(point_type) = to_earned_point_type(result.point) {
                            accessor.push_event(EventType::EarnPointEffect(point_type, pos));
                        }

                        if !result.keep_alive_as_ghost {
                            *enemy_opt = None;
                            self.decrement_alive_enemy();
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn check_shot_collision(&mut self, target: &CollBox) -> bool {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            if let Some(collbox) = shot_opt.as_mut().unwrap().get_collbox() {
                if collbox.check_collision(target) {
                    *shot_opt = None;
                    return true;
                }
            }
        }
        false
    }

    pub fn spawn(&mut self, enemy: Box<dyn Enemy>) -> bool {
        let index = calc_array_index(enemy.formation_index());
        let slot = &mut self.enemies[index];
        if slot.is_none() {
            *slot = Some(enemy);
            self.alive_enemy_count += 1;
            true
        } else {
            false
        }
    }

    pub fn spawn_captured_fighter(&mut self, pos: &Vec2I, fi: &FormationIndex) -> bool {
        let mut enemy = create_enemy(EnemyType::CapturedFighter, pos, 0, 0, fi);
        enemy.set_to_troop();
        self.spawn(enemy)
    }

    pub fn remove_enemy(&mut self, formation_index: &FormationIndex) -> bool {
        let index = calc_array_index(formation_index);
        let slot = &mut self.enemies[index];
        if slot.is_none() {
            return false;
        }
        assert!(*slot.as_ref().unwrap().formation_index() == *formation_index);
        *slot = None;
        self.decrement_alive_enemy();
        true
    }

    pub fn spawn_shot(&mut self, pos: &Vec2I, target_pos: &[Option<Vec2I>], speed: i32) {
        if self.shot_paused_count > 0 {
            return;
        }

        if let Some(index) = self.shots.iter().position(|x| x.is_none()) {
            let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
            let count = target_pos.iter().flatten().count();
            let target: &Vec2I = target_pos.iter()
                .flatten().nth(rng.gen_range(0..count)).unwrap();
            let d = target - pos;

            let limit = ANGLE * ONE * 30 / 360;
            let angle = atan2_lut(d.y, -d.x);  // 0=down
            let angle = clamp(angle, -limit, limit);

            let vel = calc_velocity(angle + ANGLE * ONE / 2, speed);
            self.shots[index] = Some(EneShot::new(pos, &vel));
        }
    }

    pub fn pause_enemy_shot(&mut self, wait: u32) {
        self.shot_paused_count = wait;
    }

    pub fn is_stationary(&self) -> bool {
        self.enemies.iter().flatten().all(|x| x.is_formation())
    }

    pub fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&dyn Enemy> {
        let index = calc_array_index(formation_index);
        self.enemies[index].as_deref()
    }

    pub fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Box<dyn Enemy>> {
        let index = calc_array_index(formation_index);
        self.enemies[index].as_mut()
    }

    fn update_enemies(&mut self, accessor: &mut impl Accessor) {
        for i in 0..self.enemies.len() {
            if let Some(enemy) = self.enemies[i].as_mut() {
                if !enemy.update(accessor) {
                    self.enemies[i] = None;
                    self.decrement_alive_enemy();
                }
            }
        }
    }

    fn update_shots(&mut self) {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            let shot = shot_opt.as_mut().unwrap();
            if !shot.update() {
                *shot_opt = None;
            }
        }
    }

    fn decrement_alive_enemy(&mut self) {
        self.alive_enemy_count -= 1;
    }

    #[cfg(debug_assertions)]
    pub fn reset_stable(&mut self) {
        self.enemies = array![_ => None; MAX_ENEMY_COUNT];
        self.shots = Default::default();
    }
}

fn calc_array_index(fi: &FormationIndex) -> usize {
    (fi.0 as usize) + (fi.1 as usize) * X_COUNT
}

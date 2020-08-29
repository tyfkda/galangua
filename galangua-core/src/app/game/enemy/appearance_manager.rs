use super::appearance_table::*;
use super::enemy::{Enemy, EnemyState};
use super::traj::Traj;

use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::ONE;

pub struct AppearanceManager {
    stage: u32,
    wait_stationary: bool,
    wait: u32,
    unit: u32,
    count: u32,
    pub(super) done: bool,
}

impl AppearanceManager {
    pub fn new(stage: u32) -> Self {
        Self {
            stage,
            wait_stationary: false,
            wait: 0,
            unit: 0,
            count: 0,
            done: true,
        }
    }

    pub fn restart(&mut self, stage: u32) {
        *self = Self::new(stage);
        self.done = false;
    }

    pub fn update(&mut self, enemies: &[Option<Enemy>]) -> Option<Vec<Enemy>> {
        if self.done {
            return None;
        }

        self.update_main(enemies)
    }

    fn update_main(&mut self, enemies: &[Option<Enemy>]) -> Option<Vec<Enemy>> {
        if self.wait_stationary {
            if !self.is_stationary(enemies) {
                return None;
            }
            self.wait_stationary = false;
        }
        if self.wait > 0 {
            self.wait -= 1;
            return None;
        }
        if self.unit >= 5 {
            self.done = true;
            return None;
        }

        let mut new_borns = Vec::new();
        let base = (self.unit * 8) as usize;
        let entry = &UNIT_TABLE[(self.stage as usize) % UNIT_TABLE.len()][self.unit as usize];
        match entry.pat {
            0 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                {
                    let index = ORDER[base + (self.count + flip * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + flip) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(entry.table, &Vec2I::new(8 * ONE, 0), true, index);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }
                {
                    let index = ORDER[base + (self.count + (1 - flip) * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (1 - flip)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(entry.table, &Vec2I::new(8 * ONE, 0), false, index);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 4 {
                    self.unit += 1;
                    self.wait_stationary = true;
                    self.wait = 10;
                    self.count = 0;
                }
            }
            1 => {
                {
                    let index = ORDER[base + (self.count / 2 + (self.count & 1) * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (self.count & 1)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(entry.table, &Vec2I::new(8 * ONE, 0), entry.flip_x, index);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 8 {
                    self.unit += 1;
                    self.wait_stationary = true;
                    self.wait = 10;
                    self.count = 0;
                }
            }
            2 => {
                {
                    let index = ORDER[base + self.count as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (self.count & 1)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(entry.table, &ZERO_VEC, entry.flip_x, index);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 8 {
                    self.unit += 1;
                    self.wait_stationary = true;
                    self.wait = 10;
                    self.count = 0;
                }
            }
            3 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                {
                    let index = ORDER[base + (self.count + flip * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + flip) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(entry.table, &Vec2I::new(8 * ONE, 0), entry.flip_x, index);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }
                {
                    let index = ORDER[base + (self.count + (1 - flip) * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (1 - flip)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(entry.table, &Vec2I::new(-8 * ONE, 0), entry.flip_x, index);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 4 {
                    self.unit += 1;
                    self.wait_stationary = true;
                    self.wait = 10;
                    self.count = 0;
                }
            }

            _ => {
                self.done = true;
                return None;
            }
        }

        Some(new_borns)
    }

    fn is_stationary(&self, enemies: &[Option<Enemy>]) -> bool {
        enemies.iter().flat_map(|x| x).all(|x| x.get_state() == EnemyState::Formation)
    }
}

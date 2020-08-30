use super::appearance_table::*;
use super::enemy::{Enemy, EnemyState, EnemyType};
use super::traj::Traj;
use super::traj_command::TrajCommand;
use super::FormationIndex;

use crate::app::util::unsafe_util::extend_lifetime;
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::ONE;

#[derive(Debug)]
struct Info {
    time: u32,
    enemy_type: EnemyType,
    fi: FormationIndex,
    offset: Vec2I,
    flip_x: bool,
    traj_table: &'static [TrajCommand],
}

impl Info {
    pub fn new(time: u32, enemy_type: EnemyType, fi: FormationIndex, offset: Vec2I, flip_x: bool,
               traj_table: &'static [TrajCommand]) -> Self {
        Self { time, enemy_type, fi, offset, flip_x, traj_table }
    }
}

pub struct AppearanceManager {
    stage: u32,
    wait_stationary: bool,
    wait: u32,
    unit: u32,
    time: u32,
    pub(super) done: bool,
    orders: Vec<Info>,
    orders_ptr: &'static [Info],
}

impl AppearanceManager {
    pub fn new(stage: u32) -> Self {
        Self {
            stage,
            wait_stationary: false,
            wait: 0,
            unit: 0,
            time: 0,
            done: true,
            orders: Vec::new(),
            orders_ptr: &[],
        }
    }

    pub fn restart(&mut self, stage: u32) {
        *self = Self::new(stage);
        self.done = false;
        self.orders.clear();
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

        if self.orders.is_empty() {
            self.set_orders();
            // orders is owned by vec, so it lives as long as self and not worry about that.
            self.orders_ptr = unsafe { extend_lifetime(&self.orders) };

            self.time = 0;
        }

        if self.orders_ptr[0].time < self.time {
            return None
        }

        let mut new_borns = Vec::new();
        while self.orders_ptr[0].time == self.time {
            let p = &self.orders_ptr[0];
            let mut enemy = Enemy::new(p.enemy_type, &ZERO_VEC, 0, 0);

            let traj = Traj::new(p.traj_table, &p.offset, p.flip_x, p.fi);
            enemy.set_traj(traj);
            enemy.formation_index = p.fi;

            new_borns.push(enemy);

            self.orders_ptr = &self.orders_ptr[1..];
            if self.orders_ptr.is_empty() {
                break;
            }
        }

        self.time += 1;
        if self.orders_ptr.is_empty() {
            self.orders_ptr = &[];
            self.orders.clear();

            self.unit += 1;
            self.wait_stationary = true;
            self.wait = 10;
            self.time = 0;
        }

        Some(new_borns)
    }

    fn set_orders(&mut self) {
        let base = self.unit * 8;
        let entry = &UNIT_TABLE[(self.stage as usize) % UNIT_TABLE.len()][self.unit as usize];
        let enemy_types = &ENEMY_TYPE_TABLE[(self.unit * 2) as usize ..
                                            (self.unit * 2) as usize + 2];

        match entry.pat {
            0 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                let mut time = 0;
                for count in 0..4 {
                    {
                        let fi = ORDER[(base + (count + flip * 4)) as usize];
                        let enemy_type = enemy_types[flip as usize];
                        self.orders.push(Info::new(time, enemy_type, fi, Vec2I::new(8 * ONE, 0),
                                                   true, entry.table))
                    }
                    {
                        let fi = ORDER[(base + (count + (1 - flip) * 4)) as usize];
                        let enemy_type = enemy_types[(1 - flip) as usize];
                        self.orders.push(Info::new(time, enemy_type, fi, Vec2I::new(8 * ONE, 0),
                                                   false,entry.table));
                    }
                    time += 16 / 3;
                }
            }
            1 => {
                let mut time = 0;
                for count in 0..8 {
                    let fi = ORDER[(base + (count / 2 + (count & 1) * 4)) as usize];
                    let enemy_type = enemy_types[(count & 1) as usize];
                    self.orders.push(Info::new(time, enemy_type, fi, Vec2I::new(8 * ONE, 0),
                                               entry.flip_x, entry.table));
                    time += 16 / 3;
                }
            }
            2 => {
                let mut time = 0;
                for count in 0..8 {
                    let fi = ORDER[(base + count) as usize];
                    let enemy_type = enemy_types[(count & 1) as usize];
                    self.orders.push(Info::new(time, enemy_type, fi, ZERO_VEC, entry.flip_x,
                                               entry.table));
                    time += 16 / 3;
                }
            }
            3 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                let mut time = 0;
                for count in 0..4 {
                    {
                        let fi = ORDER[(base + (count + flip * 4)) as usize];
                        let enemy_type = enemy_types[flip as usize];
                        self.orders.push(Info::new(time, enemy_type, fi, Vec2I::new(8 * ONE, 0),
                                                   entry.flip_x, entry.table));
                    }
                    {
                        let fi = ORDER[(base + (count + (1 - flip) * 4)) as usize];
                        let enemy_type = enemy_types[(1 - flip) as usize];
                        self.orders.push(Info::new(time, enemy_type, fi, Vec2I::new(-8 * ONE, 0),
                                                   entry.flip_x, entry.table));
                    }
                    time += 16 / 3;
                }
            }

            _ => {
                self.done = true;
            }
        }
    }

    fn is_stationary(&self, enemies: &[Option<Enemy>]) -> bool {
        enemies.iter().flat_map(|x| x).all(|x| x.get_state() == EnemyState::Formation)
    }
}

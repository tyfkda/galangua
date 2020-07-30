use counted_array::counted_array;

use super::enemy::{Enemy, EnemyState, EnemyType};
use super::traj::Traj;
use super::traj_command::TrajCommand;
use super::traj_command_table::*;
use super::FormationIndex;

use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::ONE;

const fn pos(x: u8, y: u8) -> FormationIndex { FormationIndex(x, y) }

const ORDER: [FormationIndex; 40] = [
    pos(4, 2), pos(5, 2), pos(4, 3), pos(5, 3),
    pos(4, 4), pos(5, 4), pos(4, 5), pos(5, 5),

    pos(3, 1), pos(4, 1), pos(5, 1), pos(6, 1),
    pos(3, 2), pos(6, 2), pos(3, 3), pos(6, 3),

    pos(8, 2), pos(7, 2), pos(8, 3), pos(7, 3),
    pos(1, 2), pos(2, 2), pos(1, 3), pos(2, 3),

    pos(7, 4), pos(6, 4), pos(7, 5), pos(6, 5),
    pos(3, 4), pos(2, 4), pos(3, 5), pos(2, 5),

    pos(9, 4), pos(8, 4), pos(9, 5), pos(8, 5),
    pos(0, 4), pos(1, 4), pos(0, 5), pos(1, 5),
];

const ENEMY_TYPE_TABLE: [EnemyType; 2 * 5] = [
    EnemyType::Butterfly, EnemyType::Bee,
    EnemyType::Owl, EnemyType::Butterfly,
    EnemyType::Butterfly, EnemyType::Butterfly,
    EnemyType::Bee, EnemyType::Bee,
    EnemyType::Bee, EnemyType::Bee,
];

struct UnitTableEntry<'a> {
    pat: usize,
    table: &'a [TrajCommand],
    flip_x: bool,
}

counted_array!(const UNIT_TABLE: [[UnitTableEntry; 5]; _] = [
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 1, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 1, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 2, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 2, table: &COMMAND_TABLE1, flip_x: true },
    ],
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE1, flip_x: true },
    ],
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
    ],
    [
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 3, table: &COMMAND_TABLE1, flip_x: true },
    ],
]);

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
            done: false,
        }
    }

    pub fn restart(&mut self, stage: u32) {
        *self = Self::new(stage)
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

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(8 * ONE, 0), true);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }
                {
                    let index = ORDER[base + (self.count + (1 - flip) * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (1 - flip)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(8 * ONE, 0), false);
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

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(8 * ONE, 0), entry.flip_x);
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

                    let traj = Traj::new(Some(entry.table), &ZERO_VEC, entry.flip_x);
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

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(8 * ONE, 0), entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = index;

                    new_borns.push(enemy);
                }
                {
                    let index = ORDER[base + (self.count + (1 - flip) * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (1 - flip)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &ZERO_VEC, 0, 0);

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(-8 * ONE, 0), entry.flip_x);
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

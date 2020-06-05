use crate::app::enemy::enemy::{Enemy, EnemyState, EnemyType};
use crate::app::enemy::traj::Traj;
use crate::app::enemy::traj_command::TrajCommand;
use crate::app::enemy::traj_command_table::*;
use crate::framework::types::Vec2I;
use crate::util::math::ONE;

const ORDER: [u32; 40] = [
    1 * 16 + 4,    1 * 16 + 5,    2 * 16 + 4,    2 * 16 + 5,
    3 * 16 + 4,    3 * 16 + 5,    4 * 16 + 4,    4 * 16 + 5,

    0 * 16 + 3,    0 * 16 + 4,    0 * 16 + 5,    0 * 16 + 6,
    1 * 16 + 3,    1 * 16 + 6,    2 * 16 + 3,    2 * 16 + 6,

    1 * 16 + 8,    1 * 16 + 7,    2 * 16 + 8,    2 * 16 + 7,
    1 * 16 + 1,    1 * 16 + 2,    2 * 16 + 1,    2 * 16 + 2,

    3 * 16 + 7,    3 * 16 + 6,    4 * 16 + 7,    4 * 16 + 6,
    3 * 16 + 3,    3 * 16 + 2,    4 * 16 + 3,    4 * 16 + 2,

    3 * 16 + 9,    3 * 16 + 8,    4 * 16 + 9,    4 * 16 + 8,
    3 * 16 + 0,    3 * 16 + 1,    4 * 16 + 0,    4 * 16 + 1,
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

const UNIT_TABLE: [[UnitTableEntry; 5]; 3] = [
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
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE2, flip_x: true },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: false },
        UnitTableEntry { pat: 0, table: &COMMAND_TABLE1, flip_x: true },
    ],
];

pub struct AppearanceManager {
    stage: u32,
    wait_stationary: bool,
    wait: u32,
    unit: u32,
    count: u32,
    pub done: bool,
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
        let zero = Vec2I::new(0, 0);
        let base = (self.unit * 8) as usize;
        let entry = &UNIT_TABLE[(self.stage as usize) % UNIT_TABLE.len()][self.unit as usize];
        match entry.pat {
            0 => {
                {
                    let i = ORDER[base + self.count as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2) as usize];
                    let mut enemy = Enemy::new(enemy_type, &zero, 0, 0);

                    let traj = Traj::new(Some(entry.table), &zero, true ^ entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    new_borns.push(enemy);
                }
                {
                    let i = ORDER[base + self.count as usize + 4];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + 1) as usize];
                    let mut enemy = Enemy::new(enemy_type, &zero, 0, 0);

                    let traj = Traj::new(Some(entry.table), &zero, false ^ entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

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
                    let i = ORDER[base + (self.count / 2 + (self.count & 1) * 4) as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (self.count & 1)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &zero, 0, 0);

                    let traj = Traj::new(Some(entry.table), &zero, entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

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
                    let i = ORDER[base + self.count as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + (self.count & 1)) as usize];
                    let mut enemy = Enemy::new(enemy_type, &zero, 0, 0);

                    let traj = Traj::new(Some(entry.table), &zero, entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

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
                {
                    let i = ORDER[base + self.count as usize];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2) as usize];
                    let mut enemy = Enemy::new(enemy_type, &zero, 0, 0);

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(8 * ONE, 0), entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    new_borns.push(enemy);
                }
                {
                    let i = ORDER[base + self.count as usize + 4];
                    let enemy_type = ENEMY_TYPE_TABLE[(self.unit * 2 + 1) as usize];
                    let mut enemy = Enemy::new(enemy_type, &zero, 0, 0);

                    let traj = Traj::new(Some(entry.table), &Vec2I::new(-8 * ONE, 0), entry.flip_x);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

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
        enemies.iter().flat_map(|x| x).all(|x| x.state == EnemyState::Formation)
    }
}

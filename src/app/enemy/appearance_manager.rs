use super::EnemyManager;
use super::enemy::{Enemy, EnemyType};
use super::traj::Traj;
use super::traj_command_table::*;
use super::super::super::util::types::Vec2I;

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

pub struct AppearanceManager {
    wait: u32,
    unit: u32,
    count: u32,
    done: bool,
}

impl AppearanceManager {
    pub fn new() -> AppearanceManager {
        AppearanceManager {
            wait: 0,
            unit: 0,
            count: 0,
            done: false,
        }
    }

    pub fn update(&mut self, enemy_manager: &mut EnemyManager) -> bool {
        if self.done {
            return false;
        }

        self.update_main(enemy_manager)
    }

    fn update_main(&mut self, enemy_manager: &mut EnemyManager) -> bool {
        if self.wait > 0 {
            self.wait -= 1;
            return false;
        }

        let zero = Vec2I::new(0, 0);
        match self.unit {
            0 => {
                let base = (self.unit * 8) as usize;
                {
                    let i = ORDER[base + self.count as usize];

                    let enemy_type = EnemyType::Butterfly;
                    let mut enemy = Enemy::new(enemy_type, zero, 0, 0);

                    let traj = Traj::new(Some(&COMMAND_TABLE1), zero, true);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    enemy_manager.spawn(enemy);
                }
                {
                    let i = ORDER[base + self.count as usize + 4];
                    let enemy_type = EnemyType::Bee;
                    let mut enemy = Enemy::new(enemy_type, zero, 0, 0);

                    let traj = Traj::new(Some(&COMMAND_TABLE1), zero, false);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    enemy_manager.spawn(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 4 {
                    self.unit += 1;
                    self.wait = 200;
                    self.count = 0;
                }
            },
            1 => {
                let base = (self.unit * 8) as usize;
                {
                    let i = ORDER[base + (self.count / 2 + (self.count & 1) * 4) as usize];

                    let enemy_type = if (self.count & 1) == 0 { EnemyType::Owl } else { EnemyType::Butterfly };
                    let mut enemy = Enemy::new(enemy_type, zero, 0, 0);

                    let traj = Traj::new(Some(&COMMAND_TABLE2), zero, false);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    enemy_manager.spawn(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 8 {
                    self.unit += 1;
                    self.wait = 200;
                    self.count = 0;
                }
            },
            2 => {
                let base = (self.unit * 8) as usize;
                {
                    let i = ORDER[base + self.count as usize];

                    let enemy_type = EnemyType::Butterfly;
                    let mut enemy = Enemy::new(enemy_type, zero, 0, 0);

                    let traj = Traj::new(Some(&COMMAND_TABLE2), zero, true);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    enemy_manager.spawn(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 8 {
                    self.unit += 1;
                    self.wait = 200;
                    self.count = 0;
                }
            },
            3 => {
                let base = (self.unit * 8) as usize;
                {
                    let i = ORDER[base + self.count as usize];
                    let enemy_type = EnemyType::Bee;
                    let mut enemy = Enemy::new(enemy_type, zero, 0, 0);

                    let traj = Traj::new(Some(&COMMAND_TABLE1), zero, false);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    enemy_manager.spawn(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 8 {
                    self.unit += 1;
                    self.wait = 200;
                    self.count = 0;
                }
            },
            4 => {
                let base = (self.unit * 8) as usize;
                {
                    let i = ORDER[base + self.count as usize];
                    let enemy_type = EnemyType::Bee;
                    let mut enemy = Enemy::new(enemy_type, zero, 0, 0);

                    let traj = Traj::new(Some(&COMMAND_TABLE1), zero, true);
                    enemy.set_traj(traj);
                    enemy.formation_index = i as usize;

                    enemy_manager.spawn(enemy);
                }

                self.wait = 16 / 3;
                self.count += 1;
                if self.count >= 8 {
                    self.unit += 1;
                    self.wait = 150;
                    self.count = 0;
                }
            },
            _ => {
                self.done = true;
                return true
            },
        }
        false
    }
}

use rand::Rng;

use crate::app::enemy::enemy::{Enemy, EnemyState, EnemyType};
use crate::app::enemy::Accessor;

const MAX_ATTACKER_COUNT: usize = 1;

pub struct AttackManager {
    enable: bool,
    wait: u32,
    attackers: [Option<usize>; MAX_ATTACKER_COUNT],
    player_captured: bool,
}

impl AttackManager {
    pub fn new() -> Self {
        Self {
            enable: false,
            wait: 0,
            attackers: Default::default(),
            player_captured: false,
        }
    }

    pub fn restart(&mut self, cont: bool) {
        self.enable = false;
        self.wait = 0;
        self.attackers = Default::default();
        if !cont {
            self.player_captured = false;
        }
    }

    pub fn set_enable(&mut self, value: bool) {
        self.enable = value;
    }

    pub fn set_capture_state(&mut self, value: bool) {
        self.player_captured = value;
    }

    pub fn update(&mut self, enemies: &mut [Option<Enemy>], accessor: &mut dyn Accessor) {
        if !self.enable {
            return;
        }

        self.check_liveness(enemies);

        if self.wait > 0 {
            self.wait -= 1;
            return;
        }

        if let Some(slot) = self.attackers.iter_mut().find(|x| x.is_none()) {
            let alive_indices = (0..enemies.len())
                .filter(|&i| enemies[i].is_some()).collect::<Vec<usize>>();
            let count = alive_indices.len();
            if count > 0 {
                let mut rng = rand::thread_rng();
//                let index = rng.gen_range(0, count);
let mut index = 0;
for _i in 0..100 {
    index = rng.gen_range(0, count);
    if enemies[alive_indices[index]].as_ref().unwrap().enemy_type == EnemyType::Owl {
        break;
    }
}

                let no = alive_indices[index];
                *slot = Some(no);

                let mut enemies = enemies.iter_mut()
                    .flat_map(|x| x)
                    .filter(|x| x.state == EnemyState::Formation);
                let enemy = &mut enemies.nth(index).unwrap();
                let capture_attack = enemy.enemy_type == EnemyType::Owl &&
                    !self.player_captured &&
                    !accessor.is_player_dual();
                enemy.set_attack(capture_attack, accessor);

                self.wait = 60 * 2;
            }
        }
    }

    pub fn check_liveness(&mut self, enemies: &mut [Option<Enemy>]) {
        for attacker_opt in self.attackers.iter_mut().filter(|x| x.is_some()) {
            let index = attacker_opt.unwrap();
            if let Some(enemy) = &enemies[index] {
                if enemy.state == EnemyState::Formation {
                    *attacker_opt = None;
                }
            } else {
                *attacker_opt = None;
            }
        }
    }
}

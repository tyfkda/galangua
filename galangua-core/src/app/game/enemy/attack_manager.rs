use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::enemy::{Enemy, EnemyState, EnemyType};
use super::Accessor;

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

    pub fn restart(&mut self, _stage: u32) {
        *self = Self::new();
    }

    pub fn set_enable(&mut self, value: bool) {
        self.enable = value;
    }

    pub fn is_no_attacker(&self) -> bool {
        self.attackers.iter().all(|x| x.is_none())
    }

    pub fn set_capture_state(&mut self, value: bool) {
        self.player_captured = value;
    }

    pub fn update(&mut self, enemies: &mut [Option<Enemy>], accessor: &mut dyn Accessor) {
        self.check_liveness(enemies);

        if self.wait > 0 {
            self.wait -= 1;
            return;
        }

        if !self.enable {
            return;
        }

        if let Some(slot) = self.attackers.iter_mut().find(|x| x.is_none()) {
            let candidate_indices = (0..enemies.len())
                .filter(|&i| enemies[i].as_ref().map_or(false, |e| e.get_state() == EnemyState::Formation))
                .collect::<Vec<usize>>();
            let count = candidate_indices.len();
            if count > 0 {
                let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
//                let index = rng.gen_range(0, count);
let mut index = 0;
for _i in 0..100 {
    index = rng.gen_range(0, count);
    if enemies[candidate_indices[index]].as_ref().unwrap().enemy_type == EnemyType::Owl {
        break;
    }
}

                let no = candidate_indices[index];
                *slot = Some(no);

                let enemy = &mut enemies[no].as_mut().unwrap();
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
                if enemy.get_state() == EnemyState::Formation {
                    *attacker_opt = None;
                }
            } else {
                *attacker_opt = None;
            }
        }
    }
}

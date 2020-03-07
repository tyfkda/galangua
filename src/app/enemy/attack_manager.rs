use rand::Rng;
use super::enemy::{Enemy, EnemyState};

const MAX_ATTACKER_COUNT: usize = 1;

pub struct AttackManager {
    enable: bool,
    wait: u32,
    attackers: [Option<usize>; MAX_ATTACKER_COUNT],
}

impl AttackManager {
    pub fn new() -> AttackManager {
        AttackManager {
            enable: false,
            wait: 0,
            attackers: Default::default(),
        }
    }

    pub fn restart(&mut self) {
        self.enable = false;
        self.wait = 0;
        self.attackers = Default::default();
    }

    pub fn set_enable(&mut self, value: bool) {
        if self.enable != value {
            self.enable = value;
            if self.enable {
                self.wait = 30;
            }
        }
    }

    pub fn update(&mut self, enemies: &mut [Option<Enemy>]) {
        if !self.enable {
            return;
        }

        self.check_liveness(enemies);

        if self.wait > 0 {
            self.wait -= 1;
            return;
        }

        if let Some(slot) = self.attackers.iter_mut().find(|x| x.is_none()) {
            let alive_indices = (0 .. enemies.len()).filter(|&i| enemies[i].is_some()).collect::<Vec<usize>>();
            let count = alive_indices.len();

            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0, count);
            let no = alive_indices[index];
            *slot = Some(no);

            let mut enemies = enemies.iter_mut().flat_map(|x| x).filter(|x| x.state == EnemyState::Formation);
            let enemy = &mut enemies.nth(index).unwrap();
            enemy.set_attack();

            self.wait = 60 * 2;
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

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::enemy::{Enemy, EnemyState, EnemyType};
use super::{Accessor, FormationIndex};
use crate::app::util::unsafe_util::peep;

const MAX_ATTACKER_COUNT: usize = 1;

pub struct AttackManager {
    enable: bool,
    wait: u32,
    attackers: [Option<FormationIndex>; MAX_ATTACKER_COUNT],
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

    pub fn update(&mut self, accessor: &mut dyn Accessor) {
        self.check_liveness(accessor);

        if self.wait > 0 {
            self.wait -= 1;
            return;
        }

        if !self.enable {
            return;
        }

        if let Some(slot) = self.attackers.iter_mut().find(|x| x.is_none()) {
            let enemies = accessor.get_enemies();
            let candidates = enemies.iter()
                .flat_map(|x| x.as_ref())
                .filter(|e| e.get_state() == EnemyState::Formation)
                .collect::<Vec<&Enemy>>();
            let count = candidates.len();
            if count > 0 {
                let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
//                let index = rng.gen_range(0, count);
let mut index = 0;
for _i in 0..100 {
    index = rng.gen_range(0, count);
    if candidates[index].enemy_type == EnemyType::Owl {
        break;
    }
}

                let candidate = candidates[index];
                let formation_index = candidate.formation_index;
                *slot = Some(formation_index);

                let capture_attack = candidate.enemy_type == EnemyType::Owl &&
                    !self.player_captured &&
                    !accessor.is_player_dual();
                let enemy = {
                    let accessor = unsafe { peep(accessor) };
                    accessor.get_enemy_at_mut(&formation_index).unwrap()
                };
                enemy.set_attack(capture_attack, accessor);

                self.wait = 60 * 2;
            }
        }
    }

    pub fn check_liveness(&mut self, accessor: &mut dyn Accessor) {
        for attacker_opt in self.attackers.iter_mut().filter(|x| x.is_some()) {
            let formation_index = attacker_opt.as_ref().unwrap();
            if let Some(enemy) = accessor.get_enemy_at(formation_index) {
                if enemy.get_state() == EnemyState::Formation {
                    *attacker_opt = None;
                }
            } else {
                *attacker_opt = None;
            }
        }
    }
}

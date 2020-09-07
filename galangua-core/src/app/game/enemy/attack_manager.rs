use array_macro::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::enemy::{EnemyState, EnemyType};
use super::formation::{X_COUNT, Y_COUNT};
use super::{Accessor, FormationIndex};
use crate::app::game::{CaptureState, EventQueue, EventType};
use crate::app::util::unsafe_util::peep;

const MAX_ATTACKER_COUNT: usize = 3;
const WAIT: u32 = 30;

pub struct AttackManager {
    enable: bool,
    wait: u32,
    attackers: [Option<FormationIndex>; MAX_ATTACKER_COUNT],
    cycle: u32,
}

impl AttackManager {
    pub fn new() -> Self {
        Self {
            enable: false,
            wait: 0,
            attackers: Default::default(),
            cycle: 0,
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

    pub fn update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        self.check_liveness(accessor);

        if self.wait > 0 {
            self.wait -= 1;
            return;
        }

        if !self.enable {
            return;
        }

        if let Some(slot_index) = self.attackers.iter().position(|x| x.is_none()) {
            if let Some((formation_index, capture_attack)) = self.pick_attacker(accessor) {
                self.attackers[slot_index] = Some(formation_index);
                if capture_attack {
                    event_queue.push(EventType::StartCaptureAttack(formation_index));
                }
            }
            self.wait = WAIT;
            self.cycle += 1;
        }
    }

    fn check_liveness(&mut self, accessor: &mut dyn Accessor) {
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

    fn pick_attacker(&mut self, accessor: &mut dyn Accessor) -> Option<(FormationIndex, bool)> {
        let candidates = self.enum_sides(accessor);
        let fi = match self.cycle % 3 {
            2 => {
                self.pick_random(&candidates, &mut [1])
                    .or_else(|| self.pick_captured_fighter(accessor))
            }
            0 | 1 | _ => {
                self.pick_random(&candidates, &mut [2, 3, 4, 5])
            }
        };
        if let Some(fi) = fi {
            let enemy = {
                let accessor = unsafe { peep(accessor) };
                accessor.get_enemy_at_mut(&fi).unwrap()
            };

            let capture_attack = enemy.enemy_type == EnemyType::Owl &&
                (self.cycle / 3) & 1 != 0 &&
                accessor.capture_state() == CaptureState::NoCapture &&
                !accessor.is_player_dual();
            enemy.set_attack(capture_attack, accessor);

            Some((fi, capture_attack))
        } else {
            None
        }
    }

    fn pick_random(&mut self, candidates: &[Option<[u8; 2]>; Y_COUNT], rows: &mut [u32]) -> Option<FormationIndex> {
        let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
        rows.shuffle(&mut rng);
        for &row in rows.iter() {
            if let Some(pos) = candidates[row as usize] {
                let index = rng.gen_range(0, 2);
                return Some(FormationIndex(pos[index], row as u8));
            }
        }
        None
    }

    fn enum_sides(&mut self, accessor: &mut dyn Accessor) -> [Option<[u8; 2]>; Y_COUNT] {
        array![|i| {
            let left = (0..X_COUNT).find_map(|j| {
                let fi = FormationIndex(j as u8, i as u8);
                if let Some(enemy) = accessor.get_enemy_at(&fi) {
                    if enemy.get_state() == EnemyState::Formation {
                        return Some(j);
                    }
                }
                None
            });

            if let Some(l) = left {
                let r = ((l as usize)..X_COUNT).rev().find_map(|j| {
                    let fi = FormationIndex(j as u8, i as u8);
                    if let Some(enemy) = accessor.get_enemy_at(&fi) {
                        if enemy.get_state() == EnemyState::Formation {
                            return Some(j);
                        }
                    }
                    None
                }).unwrap_or(l);
                Some([l as u8, r as u8])
            } else {
                None
            }
        }; Y_COUNT]
    }

    fn pick_captured_fighter(&mut self, accessor: &mut dyn Accessor) -> Option<FormationIndex> {
        accessor.captured_fighter_index().and_then(|fi| {
            if let Some(captured_fighter) = accessor.get_enemy_at(&fi) {
                if captured_fighter.get_state() == EnemyState::Formation &&
                    accessor.get_enemy_at(&FormationIndex(fi.0, fi.1 + 1)).is_none()
                {
                    return Some(fi);
                }
            }
            None
        })
    }
}

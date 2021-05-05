use array_macro::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use crate::app::game::formation_table::{X_COUNT, Y_COUNT};
use crate::app::game::FormationIndex;

const MAX_ATTACKER_COUNT: usize = 3;
const WAIT: u32 = 30;

pub trait Accessor {
    fn can_capture_attack(&self) -> bool;
    fn captured_fighter_index(&self) -> Option<FormationIndex>;
    fn is_enemy_live_at(&self, formation_index: &FormationIndex) -> bool;
    fn is_enemy_formation_at(&self, formation_index: &FormationIndex) -> bool;
}

pub struct AttackManager {
    enable: bool,
    paused: bool,
    wait: u32,
    attackers: [Option<FormationIndex>; MAX_ATTACKER_COUNT],
    cycle: u32,
}

impl Default for AttackManager {
    fn default() -> Self {
        Self {
            enable: false,
            paused: false,
            wait: 0,
            attackers: Default::default(),
            cycle: 0,
        }
    }
}

impl AttackManager {
    pub fn restart(&mut self, _stage: u16) {
        *self = Self::default();
    }

    pub fn set_enable(&mut self, value: bool) {
        self.enable = value;
    }

    pub fn pause(&mut self, value: bool) {
        self.paused = value;
    }

    pub fn is_no_attacker(&self) -> bool {
        self.attackers.iter().all(|x| x.is_none())
    }

    pub fn update<A: Accessor>(&mut self, accessor: &A) -> Option<(FormationIndex, bool)> {
        self.check_liveness(accessor);

        if self.wait > 0 {
            self.wait -= 1;
            return None;
        }

        if !self.enable || self.paused {
            return None;
        }

        let mut result: Option<(FormationIndex, bool)> = None;
        if self.attackers.iter().any(|x| x.is_none()) {
            result = self.pick_attacker(accessor);
            self.wait = WAIT;
            self.cycle += 1;
        }
        result
    }

    pub fn put_attacker(&mut self, formation_index: &FormationIndex) {
        let slot_index = self.attackers.iter().position(|x| x.is_none()).unwrap();
        self.attackers[slot_index] = Some(*formation_index);
    }

    fn check_liveness<A: Accessor>(&mut self, accessor: &A) {
        for attacker_opt in self.attackers.iter_mut().filter(|x| x.is_some()) {
            let formation_index = attacker_opt.as_ref().unwrap();
            if accessor.is_enemy_formation_at(formation_index) || !accessor.is_enemy_live_at(formation_index) {
                *attacker_opt = None;
            }
        }
    }

    fn pick_attacker<A: Accessor>(&mut self, accessor: &A) -> Option<(FormationIndex, bool)> {
        let candidates = self.enum_sides(accessor);
        match self.cycle % 3 {
            2 => {
                self.pick_random(&candidates, &mut [1])
                    .map(|fi| (fi, (self.cycle / 3) & 1 != 0))
                    .or_else(|| {
                        self.pick_captured_fighter_as_attacker(accessor)
                            .map(|fi| (fi, false))
                    })
            }
            /*0 | 1 |*/ _ => {
                self.pick_random(&candidates, &mut [2, 3, 4, 5])
                    .map(|fi| (fi, false))
            }
        }.map(|(fi, capture_attack)| {
            let capture_attack = capture_attack && accessor.can_capture_attack();
            (fi, capture_attack)
        })
    }

    fn pick_random(&mut self, candidates: &[Option<[u8; 2]>; Y_COUNT], rows: &mut [u32]) -> Option<FormationIndex> {
        let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
        rows.shuffle(&mut rng);
        rows.iter()
            .find_map(|&row| candidates[row as usize].map(|pos| (pos, row)))
            .map(|(pos, row)| {
                let index = rng.gen_range(0, 2);
                FormationIndex(pos[index], row as u8)
            })
    }

    fn enum_sides<A: Accessor>(&mut self, accessor: &A) -> [Option<[u8; 2]>; Y_COUNT] {
        array![i => {
            let is_formation = |j| -> Option<usize> {
                let fi = FormationIndex(j as u8, i as u8);
                if accessor.is_enemy_formation_at(&fi) {
                    Some(j)
                } else {
                    None
                }
            };

            (0..X_COUNT)
                .find_map(is_formation)
                .map(|l| {
                    let r = ((l as usize)..X_COUNT).rev()
                        .find_map(is_formation)
                        .unwrap_or(l);
                    [l as u8, r as u8]
                })
        }; Y_COUNT]
    }

    fn pick_captured_fighter_as_attacker<A: Accessor>(&mut self, accessor: &A) -> Option<FormationIndex> {
        accessor.captured_fighter_index()
            .and_then(|fi| {
                if accessor.is_enemy_formation_at(&fi) &&
                    !accessor.is_enemy_live_at(&FormationIndex(fi.0, fi.1 + 1))
                {
                    Some(fi)
                } else {
                    None
                }
            })
    }
}

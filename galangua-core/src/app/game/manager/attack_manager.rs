use array_macro::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::formation::{X_COUNT, Y_COUNT};
use super::{CaptureState, EventType};

use crate::app::game::enemy::{Accessor, FormationIndex};
use crate::app::util::unsafe_util::peep;

const MAX_ATTACKER_COUNT: usize = 3;
const WAIT: u32 = 30;

pub struct AttackManager {
    enable: bool,
    paused: bool,
    wait: u32,
    attackers: [Option<FormationIndex>; MAX_ATTACKER_COUNT],
    cycle: u32,
}

impl AttackManager {
    pub fn new() -> Self {
        Self {
            enable: false,
            paused: false,
            wait: 0,
            attackers: Default::default(),
            cycle: 0,
        }
    }

    pub fn restart(&mut self, _stage: u16) {
        *self = Self::new();
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

    pub fn update<A: Accessor>(&mut self, accessor: &mut A) {
        self.check_liveness(accessor);

        if self.wait > 0 {
            self.wait -= 1;
            return;
        }

        if !self.enable || self.paused {
            return;
        }

        if let Some(slot_index) = self.attackers.iter().position(|x| x.is_none()) {
            if let Some((formation_index, capture_attack)) = self.pick_attacker(accessor) {
                self.attackers[slot_index] = Some(formation_index);
                if capture_attack {
                    accessor.push_event(EventType::StartCaptureAttack(formation_index));
                }
            }
            self.wait = WAIT;
            self.cycle += 1;
        }
    }

    fn check_liveness<A: Accessor>(&mut self, accessor: &mut A) {
        for attacker_opt in self.attackers.iter_mut().filter(|x| x.is_some()) {
            let formation_index = attacker_opt.as_ref().unwrap();
            if let Some(enemy) = accessor.get_enemy_at(formation_index) {
                if enemy.is_formation() {
                    *attacker_opt = None;
                }
            } else {
                *attacker_opt = None;
            }
        }
    }

    fn pick_attacker<A: Accessor>(&mut self, accessor: &mut A) -> Option<(FormationIndex, bool)> {
        let candidates = self.enum_sides(accessor);
        match self.cycle % 3 {
            2 => {
                self.pick_random(&candidates, &mut [1])
                    .or_else(|| self.pick_captured_fighter_as_attacker(accessor))
            }
            0 | 1 | _ => {
                self.pick_random(&candidates, &mut [2, 3, 4, 5])
            }
        }.map(|fi| {
            let enemy = {
                let accessor = unsafe { peep(accessor) };
                accessor.get_enemy_at_mut(&fi).unwrap()
            };

            let capture_attack =
                (self.cycle / 3) & 1 != 0 &&
                accessor.capture_state() == CaptureState::NoCapture;
            let capture_attacked = enemy.start_attack(capture_attack, accessor);

            (fi, capture_attacked)
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

    fn enum_sides<A: Accessor>(&mut self, accessor: &mut A) -> [Option<[u8; 2]>; Y_COUNT] {
        array![|i| {
            let is_formation = |j| -> Option<usize> {
                let fi = FormationIndex(j as u8, i as u8);
                accessor.get_enemy_at(&fi)
                    .filter(|enemy| enemy.is_formation())
                    .map(|_| j)
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

    fn pick_captured_fighter_as_attacker<A: Accessor>(&mut self, accessor: &mut A) -> Option<FormationIndex> {
        accessor.captured_fighter_index()
            .and_then(|fi| accessor.get_enemy_at(&fi)
                .map(|c| (c, fi)))  // TODO: use Option.zip.
            .filter(|(cap, fi)|
                cap.is_formation() &&
                    accessor.get_enemy_at(&FormationIndex(fi.0, fi.1 + 1)).is_none())
            .map(|(_cap, fi)| fi)
    }
}

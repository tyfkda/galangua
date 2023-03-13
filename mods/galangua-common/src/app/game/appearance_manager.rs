use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;
use std::cmp::min;

use crate::app::game::appearance_table::*;
use crate::app::game::traj::Traj;
use crate::app::game::traj_command::TrajCommand;
use crate::app::game::{EnemyType, FormationIndex};
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::ONE;
use crate::util::unsafe_util::extend_lifetime;

const ASSAULT_FORMATION_Y: u8 = 6;
const UNIT_COUNT: u32 = 5;
const STEP_WAIT: u32 = 16 / 3;

pub struct NewBorned {
    pub enemy_type: EnemyType,
    pub pos: Vec2I,
    pub angle: i32,
    pub speed: i32,
    pub fi: FormationIndex,
    pub traj: Traj,
}

impl NewBorned {
    fn new(
        enemy_type: EnemyType, pos: Vec2I, angle: i32, speed: i32,
        fi: FormationIndex, traj: Traj,
    ) -> Self {
        Self {
            enemy_type,
            pos,
            angle,
            speed,
            fi,
            traj,
        }
    }
}

#[derive(Clone)]
struct Info {
    time: u32,
    enemy_type: EnemyType,
    fi: FormationIndex,
    offset: Vec2I,
    flip_x: bool,
    traj_table: &'static [TrajCommand],
    shot_enable: bool,
}

impl Info {
    pub fn new(time: u32, enemy_type: EnemyType, fi: FormationIndex, offset: Vec2I, flip_x: bool,
               traj_table: &'static [TrajCommand]) -> Self {
        Self {
            time, enemy_type, fi, offset, flip_x, traj_table,
            shot_enable: false,
        }
    }
}

pub trait Accessor {
    fn is_stationary(&self) -> bool;
}

pub struct AppearanceManager {
    stage: u16,
    paused: bool,
    wait_stationary: bool,
    wait: u32,
    unit: u32,
    time: u32,
    pub done: bool,
    orders: Vec<Info>,
    orders_ptr: &'static [Info],
    captured_fighter: Option<FormationIndex>,
}

impl Default for AppearanceManager {
    fn default() -> Self {
        Self {
            stage: 0,
            paused: false,
            wait_stationary: false,
            wait: 0,
            unit: 0,
            time: 0,
            done: true,
            orders: Vec::new(),
            orders_ptr: &[],
            captured_fighter: None,
        }
    }
}

impl AppearanceManager {
    pub fn restart(&mut self, stage: u16, captured_fighter: Option<FormationIndex>) {
        *self = Self::default();
        self.stage = stage;
        self.done = false;
        self.captured_fighter = captured_fighter;
    }

    pub fn pause(&mut self, value: bool) {
        self.paused = value;
    }

    pub fn update(&mut self, accessor: &impl Accessor) -> Option<Vec<NewBorned>> {
        if self.done {
            return None;
        }

        self.update_main(accessor)
    }

    fn update_main(&mut self, accessor: &impl Accessor) -> Option<Vec<NewBorned>> {
        if self.wait > 0 {
            self.wait -= 1;
            return None;
        }

        if !self.paused {
            if self.wait_stationary {
                if !accessor.is_stationary() {
                    return None;
                }
                self.wait_stationary = false;
            }
            if self.unit >= UNIT_COUNT {
                self.done = true;
                return None;
            }

            if self.orders.is_empty() {
                self.set_orders();
                // orders is owned by vec, so it lives as long as self and not worry about that.
                self.orders_ptr = unsafe { extend_lifetime(&self.orders) };

                self.time = 0;
            }
        }

        if self.orders.is_empty() || self.orders_ptr[0].time < self.time {
            return None;
        }

        let mut new_borns: Vec<NewBorned> = Vec::new();
        while self.orders_ptr[0].time == self.time {
            let p = &self.orders_ptr[0];
            let mut traj = Traj::new(p.traj_table, &p.offset, p.flip_x, p.fi);
            traj.shot_enable = p.shot_enable;

            let enemy = NewBorned::new(p.enemy_type, ZERO_VEC, 0, 0, p.fi, traj);
            new_borns.push(enemy);

            self.orders_ptr = &self.orders_ptr[1..];
            if self.orders_ptr.is_empty() {
                break;
            }
        }

        self.time += 1;
        if self.orders_ptr.is_empty() {
            self.orders_ptr = &[];
            self.orders.clear();

            self.unit += 1;
            self.wait_stationary = true;
            self.wait = 10;
            self.time = 0;
        }

        Some(new_borns)
    }

    fn set_orders(&mut self) {
        self.create_orders();
        self.set_shot_enables();
    }

    fn create_orders(&mut self) {
        let base = self.unit * 8;
        let entry = &UNIT_TABLE[(self.stage as usize) % UNIT_TABLE.len()][self.unit as usize];
        let assault_count = ASSAULT_TABLE[min(self.stage as usize, ASSAULT_TABLE.len() - 1)]
            [self.unit as usize] as usize;

        let div = match entry.pat {
            0 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                for count in 0..8 {
                    let side = count & 1;
                    let fi = ORDER[(base + (count / 2 + (side ^ flip) * 4)) as usize];
                    let info = self.create_info(fi, count);
                    self.orders.push(info);
                }
                2
            }
            1 => {
                for count in 0..8 {
                    let fi = ORDER[(base + (count / 2 + (count & 1) * 4)) as usize];
                    let info = self.create_info(fi, count);
                    self.orders.push(info);
                }
                1
            }
            2 => {
                for count in 0..8 {
                    let fi = ORDER[(base + count) as usize];
                    let info = self.create_info(fi, count);
                    self.orders.push(info);
                }
                1
            }
            3 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                for count in 0..8 {
                    let side = count & 1;
                    let fi = ORDER[(base + (count / 2 + (side ^ flip) * 4)) as usize];
                    let info = self.create_info(fi, count);
                    self.orders.push(info);
                }
                2
            }

            _ => { panic!("Illegal"); }
        };

        if assault_count > 0 {
            let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
            for i in 0..assault_count * 2 {
                let lr = i & 1;
                let n = self.orders.len() / 2;
                let index = rng.gen_range(0..(n + 1));
                self.orders.push(self.orders[lr].clone());
                // Shift
                for j in 0..(n - index) {
                    self.orders[(n - j) * 2 + lr] = self.orders[(n - j - 1) * 2 + lr].clone();
                }

                let fi = gen_assault_index(i as u8);
                let ins = index * 2 + lr;
                self.orders[ins] = self.create_info(fi, ins as u32);
            }

            recalc_order_time(&mut self.orders, STEP_WAIT, div);
        }

        if self.unit == UNIT_COUNT - 1 {
            if let Some(fi) = self.captured_fighter {
                let mut info = self.create_info(fi, self.orders.len() as u32);
                info.enemy_type = EnemyType::CapturedFighter;
                self.orders.push(info);
            }
        }
    }

    fn create_info(&self, fi: FormationIndex, count: u32) -> Info {
        let entry = &UNIT_TABLE[(self.stage as usize) % UNIT_TABLE.len()][self.unit as usize];
        let enemy_types = &ENEMY_TYPE_TABLE[(self.unit * 2) as usize ..
                                            (self.unit * 2) as usize + 2];
        match entry.pat {
            0 => {
                let flip = if entry.flip_x { 1 } else { 0 };
                let side = count & 1;
                let enemy_type = enemy_types[(side ^ flip) as usize];
                let time = (count / 2) * STEP_WAIT;
                Info::new(time, enemy_type, fi, Vec2I::new(8 * ONE, 0), side == 0, entry.table)
            }
            1 => {
                let enemy_type = enemy_types[(count & 1) as usize];
                let time = count * STEP_WAIT;
                Info::new(time, enemy_type, fi, Vec2I::new(8 * ONE, 0), entry.flip_x, entry.table)
            }
            2 => {
                let enemy_type = enemy_types[(count & 1) as usize];
                let time = count * STEP_WAIT;
                Info::new(time, enemy_type, fi, ZERO_VEC, entry.flip_x, entry.table)
            }
            /*3 |*/ _ => {
                let flip = if entry.flip_x { 1 } else { 0 };
                let side = count & 1;
                let enemy_type = enemy_types[(side ^ flip) as usize];
                let flag = 1 - (side as i32) * 2;
                let time = (count / 2) * STEP_WAIT;
                Info::new(time, enemy_type, fi, Vec2I::new(flag * 8 * ONE, 0), entry.flip_x, entry.table)
            }
        }
    }

    fn set_shot_enables(&mut self) {
        let count = SHOT_ENABLE_TABLE[min(self.stage as usize, SHOT_ENABLE_TABLE.len() - 1)][self.unit as usize];
        if count == 0 {
            return;
        }

        let orders = &mut self.orders;
        let mut nums = Vec::with_capacity(orders.len());
        for i in 0..orders.len() {
            nums.push(i);
        }
        let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
        nums.partial_shuffle(&mut rng, count as usize);

        for i in 0..count {
            orders[nums[i as usize]].shot_enable = true;
        }
    }
}

fn gen_assault_index(assault_count: u8) -> FormationIndex {
    let x = assault_count;
    FormationIndex(x, ASSAULT_FORMATION_Y)
}

fn recalc_order_time(orders: &mut [Info], step_wait: u32, div: u32) {
    for (i, order) in orders.iter_mut().enumerate() {
        order.time = step_wait * (i as u32 / div);
    }
}

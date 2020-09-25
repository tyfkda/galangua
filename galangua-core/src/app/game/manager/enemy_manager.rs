use array_macro::*;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::appearance_manager::AppearanceManager;
use super::attack_manager::AttackManager;
use super::formation::Formation;
use super::EventType;

use crate::app::game::effect::to_earned_point_type;
use crate::app::game::enemy::ene_shot::EneShot;
use crate::app::game::enemy::enemy::{create_enemy, Enemy};
use crate::app::game::enemy::{Accessor, EnemyType, FormationIndex};
use crate::app::util::collision::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{atan2_lut, calc_velocity, clamp, ANGLE, ONE};

const MAX_ENEMY_COUNT: usize = 70;
const MAX_SHOT_COUNT: usize = 12;
const RUSH_THRESHOLD: u32 = 5;

#[derive(PartialEq)]
enum StageState {
    APPEARANCE,
    NORMAL,
    RUSH,
}

pub struct EnemyManager {
    enemies: [Option<Box<dyn Enemy>>; MAX_ENEMY_COUNT],
    alive_enemy_count: u32,
    shots: [Option<EneShot>; MAX_SHOT_COUNT],
    shot_paused_count: u32,
    formation: Formation,
    appearance_manager: AppearanceManager,
    attack_manager: AttackManager,
    stage_state: StageState,
    frame_count: u32,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: array![None; MAX_ENEMY_COUNT],
            alive_enemy_count: 0,
            shots: Default::default(),
            shot_paused_count: 0,
            formation: Formation::new(),
            appearance_manager: AppearanceManager::new(0),
            attack_manager: AttackManager::new(),
            stage_state: StageState::APPEARANCE,
            frame_count: 0,
        }
    }

    pub fn start_next_stage(&mut self, stage: u16, captured_fighter: Option<FormationIndex>) {
        self.enemies = array![None; MAX_ENEMY_COUNT];
        self.alive_enemy_count = 0;
        self.shots = Default::default();

        self.appearance_manager.restart(stage, captured_fighter);
        self.formation.restart();
        self.attack_manager.restart(stage);
        self.shot_paused_count = 0;
        self.stage_state = StageState::APPEARANCE;
        self.frame_count = 0;
    }

    pub fn all_destroyed(&self) -> bool {
        self.appearance_manager.done && self.alive_enemy_count == 0 &&
            self.shots.iter().all(|x| x.is_none())
    }

    pub fn update<T: Accessor>(&mut self, accessor: &mut T) {
        self.frame_count = self.frame_count.wrapping_add(1);
        if self.shot_paused_count > 0 {
            self.shot_paused_count -= 1;
        }

        self.update_appearance();
        self.update_formation();
        self.update_attackers(accessor);
        self.update_enemies(accessor);
        self.update_shots();
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        let pat = ((self.frame_count >> 5) & 1) as usize;
        for enemy in self.enemies.iter().rev().flat_map(|x| x) {
            enemy.draw(renderer, pat);
        }
        for shot in self.shots.iter().flat_map(|x| x) {
            shot.draw(renderer);
        }
    }

    pub fn check_collision(&mut self, target: &CollBox) -> Option<FormationIndex> {
        self.enemies.iter().flat_map(|x| x)
            .find_map(|enemy|
                enemy.get_collbox().map(|colbox| (colbox, enemy))
                    .filter(|(colbox, _enemy)| colbox.check_collision(target))
                    .map(|(_colbox, enemy)| *enemy.formation_index()))
    }

    pub fn set_damage_to_enemy<T: Accessor>(
        &mut self, fi: &FormationIndex, power: u32, accessor: &mut T,
    ) {
        let index = calc_array_index(fi);
        if let Some(enemy) = self.enemies[index].as_mut() {
            let pos = *enemy.pos();
            let result = enemy.set_damage(power, accessor);

            if result.point > 0 {
                accessor.push_event(EventType::AddScore(result.point));

                if let Some(point_type) = to_earned_point_type(result.point) {
                    accessor.push_event(EventType::EarnPointEffect(point_type, pos));
                }

                if !result.keep_alive_as_ghost {
                    self.enemies[index] = None;
                    self.decrement_alive_enemy();
                }
            }
        }
    }

    pub fn check_shot_collision(&mut self, target: &CollBox) -> Option<Vec2I> {
        self.shots.iter_mut().filter(|x| x.is_some())
            .find_map(|shot_opt| {
                shot_opt.as_mut().unwrap().get_collbox()
                    .filter(|colbox| colbox.check_collision(target))
                    .map(|_colbox| {
                        *shot_opt.take().unwrap().pos()
                    })
            })
    }

    fn update_appearance(&mut self) {
        let prev_done = self.appearance_manager.done;
        if let Some(new_borns) = self.appearance_manager.update(&self.enemies) {
            for enemy in new_borns {
                self.spawn(enemy);
            }
        }
        if !prev_done && self.appearance_manager.done {
            self.stage_state = StageState::NORMAL;
            self.formation.done_appearance();
            if self.alive_enemy_count <= RUSH_THRESHOLD {
                self.stage_state = StageState::RUSH;
            }
            self.attack_manager.set_enable(true);
        }
    }

    fn spawn(&mut self, enemy: Box<dyn Enemy>) -> bool {
        let index = calc_array_index(enemy.formation_index());
        let slot = &mut self.enemies[index];
        if slot.is_none() {
            *slot = Some(enemy);
            self.alive_enemy_count += 1;
            true
        } else {
            false
        }
    }

    pub fn spawn_captured_fighter(&mut self, pos: &Vec2I, fi: &FormationIndex) -> bool {
        let mut enemy = create_enemy(EnemyType::CapturedFighter, &pos, 0, 0, fi);
        enemy.set_to_troop();
        self.spawn(enemy)
    }

    pub fn remove_enemy(&mut self, formation_index: &FormationIndex) -> bool {
        if let Some(slot) = self.enemies.iter_mut().filter(|x| x.is_some())
            .find(|x| *x.as_ref().unwrap().formation_index() == *formation_index)
        {
            *slot = None;
            self.decrement_alive_enemy();
            true
        } else {
            false
        }
    }

    fn update_formation(&mut self) {
        self.formation.update();
    }

    fn update_attackers<T: Accessor>(&mut self, accessor: &mut T) {
        self.attack_manager.update(accessor);
    }

    fn update_enemies<T: Accessor>(&mut self, accessor: &mut T) {
        //for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
        for i in 0..self.enemies.len() {
            if let Some(enemy) = self.enemies[i].as_mut() {
                if !enemy.update(accessor) {
                    self.enemies[i] = None;
                    self.decrement_alive_enemy();
                }
            }
        }
    }

    fn decrement_alive_enemy(&mut self) {
        self.alive_enemy_count -= 1;
        if self.alive_enemy_count <= RUSH_THRESHOLD && self.appearance_manager.done {
            self.stage_state = StageState::RUSH;
        }
    }

    fn update_shots(&mut self) {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            let shot = shot_opt.as_mut().unwrap();
            if !shot.update() {
                *shot_opt = None;
            }
        }
    }

    pub fn pause_enemy_shot(&mut self, wait: u32) {
        self.shot_paused_count = wait;
    }

    pub fn spawn_shot(&mut self, pos: &Vec2I, target_pos: &[Option<Vec2I>], speed: i32) {
        if self.shot_paused_count > 0 {
            return;
        }

        if let Some(index) = self.shots.iter().position(|x| x.is_none()) {
            let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
            let count = target_pos.iter().flat_map(|x| x).count();
            let target: &Vec2I = target_pos.iter()
                .flat_map(|x| x).nth(rng.gen_range(0, count)).unwrap();
            let d = target - &pos;

            let limit = ANGLE * ONE * 30 / 360;
            let angle = atan2_lut(d.y, -d.x);  // 0=down
            let angle = clamp(angle, -limit, limit);

            let vel = calc_velocity(angle + ANGLE * ONE / 2, speed);
            self.shots[index] = Some(EneShot::new(&pos, &vel));
        }
    }

    pub fn pause_attack(&mut self, value: bool) {
        self.attack_manager.pause(value);
        self.appearance_manager.pause(value);
    }

    pub fn is_no_attacker(&self) -> bool {
        self.attack_manager.is_no_attacker()
    }

    pub fn get_enemies(&self) -> &[Option<Box<dyn Enemy>>] {
        &self.enemies
    }

    pub fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Box<dyn Enemy>> {
        let index = calc_array_index(formation_index);
        self.enemies[index].as_ref()
    }

    pub fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Box<dyn Enemy>> {
        let index = calc_array_index(formation_index);
        self.enemies[index].as_mut()
    }

    pub fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }

    pub fn is_rush(&self) -> bool {
        self.stage_state == StageState::RUSH
    }

    // Debug

    #[cfg(debug_assertions)]
    pub fn reset_stable(&mut self) {
        self.enemies = array![None; MAX_ENEMY_COUNT];
        self.shots = Default::default();

        let stage = 0;
        self.appearance_manager.restart(stage, None);
        self.appearance_manager.done = true;
        self.formation.restart();
        self.formation.done_appearance();
        self.attack_manager.restart(stage);
        self.attack_manager.set_enable(false);
        self.stage_state = StageState::NORMAL;

        for unit in 0..5 {
            for i in 0..8 {
                let index = super::appearance_table::ORDER[unit * 8 + i];
                let enemy_type = super::appearance_table::ENEMY_TYPE_TABLE[unit * 2 + (i / 4)];
                let pos = self.formation.pos(&index);
                let mut enemy = create_enemy(enemy_type, &pos, 0, 0, &index);
                enemy.set_to_formation();
                self.spawn(enemy);
            }
        }
    }
}

fn calc_array_index(fi: &FormationIndex) -> usize {
    (fi.0 + fi.1 * 10) as usize  // Caution!
}

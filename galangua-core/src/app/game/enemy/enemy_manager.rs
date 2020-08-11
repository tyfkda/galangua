use array_macro::*;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128Plus;

use super::appearance_manager::AppearanceManager;
use super::attack_manager::AttackManager;
use super::ene_shot::EneShot;
use super::enemy::{CaptureState, Enemy, EnemyType};
use super::formation::Formation;
use super::{Accessor, FormationIndex};

use crate::app::consts::*;
use crate::app::game::effect::EarnedPointType;
use crate::app::game::{EventQueue, EventType};
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;

const MAX_ENEMY_COUNT: usize = 64;
const MAX_SHOT_COUNT: usize = 16;

pub struct EnemyManager {
    enemies: [Option<Enemy>; MAX_ENEMY_COUNT],
    shots: [Option<EneShot>; MAX_SHOT_COUNT],
    formation: Formation,
    appearance_manager: AppearanceManager,
    wait_settle: bool,
    attack_manager: AttackManager,
    frame_count: u32,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: array![None; MAX_ENEMY_COUNT],
            shots: Default::default(),
            formation: Formation::new(),
            appearance_manager: AppearanceManager::new(0),
            wait_settle: true,
            attack_manager: AttackManager::new(),
            frame_count: 0,
        }
    }

    pub fn start_next_stage(&mut self, stage: u32) {
        self.enemies = array![None; MAX_ENEMY_COUNT];
        self.shots = Default::default();

        self.appearance_manager.restart(stage);
        self.formation.restart();
        self.attack_manager.restart(stage);
        self.wait_settle = true;
    }

    pub fn all_destroyed(&self) -> bool {
        self.appearance_manager.done && self.enemies.iter().all(|x| x.is_none())
    }

    pub fn update<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        self.frame_count = self.frame_count.wrapping_add(1);
        self.update_appearance();
        self.update_formation();
        self.update_attackers(accessor);
        self.update_enemies(accessor, event_queue);
        self.update_shots();
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let pat = ((self.frame_count >> 5) & 1) as usize;
        for enemy in self.enemies.iter().flat_map(|x| x) {
            enemy.draw(renderer, pat)?;
        }
        for shot in self.shots.iter().flat_map(|x| x) {
            shot.draw(renderer)?;
        }

        Ok(())
    }

    pub fn check_collision<T: Accessor>(
        &mut self, target: &CollBox, power: u32, accessor: &T, event_queue: &mut EventQueue,
    ) -> Option<Vec2I> {
        for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
            let enemy = enemy_opt.as_mut().unwrap();
            if let Some(colbox) = enemy.get_collbox() {
                if colbox.check_collision(target) {
                    let pos = *enemy.raw_pos();
                    let result = enemy.set_damage(power, accessor, event_queue);

                    if result.point > 0 {
                        event_queue.push(EventType::AddScore(result.point));
                        event_queue.push(EventType::SmallBomb(pos));

                        let point_type = match result.point {
                            1600 => Some(EarnedPointType::Point1600),
                            1000 => Some(EarnedPointType::Point1000),
                            800 => Some(EarnedPointType::Point800),
                            400 => Some(EarnedPointType::Point400),
                            _ => None,
                        };
                        if let Some(point_type) = point_type {
                            event_queue.push(EventType::EarnPoint(point_type, pos));
                        }
                    }

                    let capture_state = enemy.capture_state();
                    let captured_fighter_index = enemy.captured_fighter_index();

                    if result.destroyed {
                        match capture_state {
                            CaptureState::None | CaptureState::Capturing => {}
                            CaptureState::BeamTracting | CaptureState::BeamClosing => {
                                event_queue.push(EventType::EscapeCapturing);
                            }
                        }
                        if let Some(fi) = captured_fighter_index {
                            event_queue.push(EventType::RecapturePlayer(fi));
                        }
                    }

                    if result.killed {
                        *enemy_opt = None;
                    }
                    return Some(pos);
                }
            }
        }
        return None;
    }

    pub fn check_shot_collision(&mut self, target: &CollBox) -> Option<Vec2I> {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            let shot = shot_opt.as_mut().unwrap();
            if let Some(colbox) = shot.get_collbox() {
                if colbox.check_collision(target) {
                    let pos = *shot.raw_pos();
                    *shot_opt = None;
                    return Some(pos);
                }
            }
        }
        return None;
    }

    fn update_appearance(&mut self) {
        let prev_done = self.appearance_manager.done;
        if let Some(new_borns) = self.appearance_manager.update(&self.enemies) {
            for enemy in new_borns {
                self.spawn(enemy);
            }
        }
        if !prev_done && self.appearance_manager.done {
            self.formation.done_appearance();
        }
    }

    fn spawn(&mut self, enemy: Enemy) -> bool {
        if let Some(index) = self.find_slot() {
            self.enemies[index] = Some(enemy);
            true
        } else {
            false
        }
    }

    pub fn spawn_captured_fighter(&mut self, pos: &Vec2I, formation_index: &FormationIndex) -> bool {
        let mut enemy = Enemy::new(EnemyType::CapturedFighter, &pos, 0, 0);
        enemy.set_to_troop();
        enemy.formation_index = *formation_index;
        self.spawn(enemy)
    }

    pub fn remove_enemy(&mut self, formation_index: &FormationIndex) -> bool {
        if let Some(slot) = self.enemies.iter_mut().filter(|x| x.is_some())
            .find(|x| x.as_ref().unwrap().formation_index == *formation_index)
        {
            *slot = None;
            true
        } else {
            false
        }
    }

    fn update_formation(&mut self) {
        self.formation.update();
        if self.wait_settle && self.formation.is_settle() {
            self.wait_settle = false;
            self.attack_manager.set_enable(true);
        }
    }

    fn update_attackers<T: Accessor>(&mut self, accessor: &mut T) {
        self.attack_manager.update(&mut self.enemies, accessor);
    }

    fn update_enemies<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
            let enemy = enemy_opt.as_mut().unwrap();
            enemy.update(accessor, event_queue);
            if enemy.is_disappeared() {
                *enemy_opt = None;
            }
        }
    }

    fn update_shots(&mut self) {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            let shot = shot_opt.as_mut().unwrap();
            shot.update();
            if out_of_screen(&shot.pos()) {
                *shot_opt = None;
            }
        }
    }

    fn find_slot(&self) -> Option<usize> {
        self.enemies.iter().position(|x| x.is_none())
    }

    pub fn spawn_shot(&mut self, pos: &Vec2I, target_pos: &[Option<Vec2I>], speed: i32) {
        if let Some(index) = self.shots.iter().position(|x| x.is_none()) {
            let mut rng = Xoshiro128Plus::from_seed(rand::thread_rng().gen());
            let count = target_pos.iter().flat_map(|x| x).count();
            let target: &Vec2I = target_pos.iter()
                .flat_map(|x| x).nth(rng.gen_range(0, count)).unwrap();
            let d = target - &pos;
            let distance = ((d.x as f64).powi(2) + (d.y as f64).powi(2)).sqrt();
            let f = (speed as f64) / distance;
            let vel = Vec2I::new(
                ((d.x as f64) * f).round() as i32,
                ((d.y as f64) * f).round() as i32,
            );
            self.shots[index] = Some(EneShot::new(&pos, &vel));
        }
    }

    pub fn set_capture_state(&mut self, value: bool) {
        self.attack_manager.set_capture_state(value);
    }

    pub fn enable_attack(&mut self, value: bool) {
        self.attack_manager.set_enable(value);
    }

    pub fn is_no_attacker(&self) -> bool {
        self.attack_manager.is_no_attacker()
    }

    pub fn get_enemy_at(&self, formation_index: &FormationIndex) -> Option<&Enemy> {
        self.enemies.iter().flat_map(|x| x)
            .find(|enemy| enemy.formation_index == *formation_index)
    }

    pub fn get_enemy_at_mut(&mut self, formation_index: &FormationIndex) -> Option<&mut Enemy> {
        self.enemies.iter_mut().flat_map(|x| x)
            .find(|enemy| enemy.formation_index == *formation_index)
    }

    pub fn get_formation_pos(&self, formation_index: &FormationIndex) -> Vec2I {
        self.formation.pos(formation_index)
    }

    // Debug

    #[cfg(debug_assertions)]
    pub fn reset_stable(&mut self) {
        self.enemies = array![None; MAX_ENEMY_COUNT];
        self.shots = Default::default();

        let stage = 0;
        self.appearance_manager.restart(stage);
        self.appearance_manager.done = true;
        self.formation.restart();
        self.formation.done_appearance();
        self.attack_manager.restart(stage);
        self.attack_manager.set_enable(false);
        self.wait_settle = false;

        for unit in 0..5 {
            for i in 0..8 {
                let index = super::appearance_manager::ORDER[unit * 8 + i];
                let enemy_type = super::appearance_manager::ENEMY_TYPE_TABLE[unit * 2 + (i / 4)];
                let pos = self.formation.pos(&index);
                let mut enemy = Enemy::new(enemy_type, &pos, 0, 0);
                enemy.formation_index = index;
                enemy.set_to_formation();
                self.spawn(enemy);
            }
        }
    }
}

fn out_of_screen(pos: &Vec2I) -> bool {
    pos.x < -16 || pos.x > WIDTH + 16
        || pos.y < -16 || pos.y > HEIGHT + 16
}

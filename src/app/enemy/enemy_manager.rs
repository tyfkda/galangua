use array_macro::*;
use rand::Rng;

use crate::app::consts::*;
use crate::app::effect::EarnedPointType;
use crate::app::enemy::appearance_manager::AppearanceManager;
use crate::app::enemy::attack_manager::AttackManager;
use crate::app::enemy::ene_shot::EneShot;
use crate::app::enemy::enemy::{CaptureState, Enemy, EnemyState, EnemyType};
use crate::app::enemy::formation::Formation;
use crate::app::enemy::{Accessor, FormationIndex};
use crate::app::game::{EventQueue, EventType};
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::ONE;

const MAX_ENEMY_COUNT: usize = 64;
const MAX_SHOT_COUNT: usize = 16;

pub struct EnemyManager {
    enemies: [Option<Enemy>; MAX_ENEMY_COUNT],
    shots: [Option<EneShot>; MAX_SHOT_COUNT],
    formation: Formation,
    appearance_manager: AppearanceManager,
    wait_settle: bool,
    attack_manager: AttackManager,
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
        }
    }

    pub fn restart(&mut self, stage: u32) {
        for slot in self.enemies.iter_mut() {
            *slot = None;
        }
        for slot in self.shots.iter_mut() {
            *slot = None;
        }

        self.start_next_stage(stage);
    }

    pub fn start_next_stage(&mut self, stage: u32) {
        self.appearance_manager = AppearanceManager::new(stage);
        self.formation.restart();
        self.attack_manager.restart(stage > 0);
        self.wait_settle = true;
    }

    pub fn all_destroyed(&self) -> bool {
        self.appearance_manager.done && self.enemies.iter().all(|x| x.is_none())
    }

    pub fn update<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        self.update_appearance();
        self.update_formation();
        self.update_attackers(accessor);
        self.update_enemies(accessor, event_queue);
        self.update_shots();
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        for enemy in self.enemies.iter().flat_map(|x| x) {
            enemy.draw(renderer)?;
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
                    let result = enemy.set_damage(power, accessor);

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
        enemy.state = EnemyState::Troop;
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
            if enemy.is_dead() {
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
            let mut rng = rand::thread_rng();
            let count = target_pos.iter().filter(|x| x.is_some()).count();
            let target_opt: &Option<Vec2I> = target_pos.iter()
                .filter(|x| x.is_some()).nth(rng.gen_range(0, count)).unwrap();
            let target: Vec2I = target_opt.unwrap();
            let d = &(&target * ONE) - &pos;
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
}

fn out_of_screen(pos: &Vec2I) -> bool {
    pos.x < -16 || pos.x > WIDTH + 16
        || pos.y < -16 || pos.y > HEIGHT + 16
}

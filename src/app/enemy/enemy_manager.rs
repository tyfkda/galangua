use rand::Rng;
use sdl2::render::{Texture, WindowCanvas};
use std::mem::MaybeUninit;

use super::AppearanceManager;
use super::enemy::{Enemy, EnemyState};
use super::ene_shot::EneShot;
use super::formation::Formation;
use super::super::util::{CollisionResult, CollBox, Collidable};
use super::super::super::util::types::Vec2I;

const MAX_ENEMY_COUNT: usize = 64;
const MAX_SHOT_COUNT: usize = 16;

pub struct EnemyManager {
    enemies: [Option<Enemy>; MAX_ENEMY_COUNT],
    shots: [Option<EneShot>; MAX_SHOT_COUNT],
    formation: Formation,
    appearance_manager: AppearanceManager,
}

impl EnemyManager {
    pub fn new() -> EnemyManager {
        let mut enemies: [MaybeUninit<Option<Enemy>>; MAX_ENEMY_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };
        for (_i, element) in enemies.iter_mut().enumerate() {
            *element = MaybeUninit::new(None);
        }
        let enemies = unsafe { std::mem::transmute::<_, [Option<Enemy>; MAX_ENEMY_COUNT]>(enemies) };

        let mut mgr = EnemyManager {
            enemies: enemies,
            shots: Default::default(),
            formation: Formation::new(),
            appearance_manager: AppearanceManager::new(),
        };
        mgr.restart();
        mgr
    }

    pub fn restart(&mut self) {
        for slot in self.enemies.iter_mut() {
            *slot = None;
        }
        for slot in self.shots.iter_mut() {
            *slot = None;
        }

        self.appearance_manager = AppearanceManager::new();
        self.formation.restart();
    }

    pub fn spawn(&mut self, enemy: Enemy) -> bool {
        if let Some(index) = self.find_slot() {
            self.enemies[index] = Some(enemy);
            true
        } else {
            false
        }
    }

    pub fn done_appearance(&mut self) {
        self.formation.done_appearance();
    }

    pub fn update(&mut self, _player_pos: &[Option<Vec2I>]) {
        self.update_appearance();
        self.update_formation();
        self.update_enemies();
        self.update_shots();
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &mut Texture) -> Result<(), String> {
        for enemy in self.enemies.iter().flat_map(|x| x) {
            enemy.draw(canvas, texture)?;
        }
        for shot in self.shots.iter().flat_map(|x| x) {
            shot.draw(canvas, texture)?;
        }

        Ok(())
    }

    pub fn check_collision(&mut self, target: &CollBox, power: u32) -> CollisionResult {
        for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
            let enemy = enemy_opt.as_mut().unwrap();
            if enemy.get_collbox().check_collision(target) {
                let pos = enemy.pos();
                let destroyed = enemy.set_damage(power);
                if destroyed {
                    *enemy_opt = None;
                }
                return CollisionResult::Hit(pos, destroyed);
            }
        }

        return CollisionResult::NoHit;
    }

    pub fn check_shot_collision(&mut self, target: &CollBox) -> CollisionResult {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            let shot = shot_opt.as_mut().unwrap();
            if shot.get_collbox().check_collision(target) {
                let pos = shot.pos();
                *shot_opt = None;
                return CollisionResult::Hit(pos, false);
            }
        }

        return CollisionResult::NoHit;
    }

    fn update_appearance(&mut self) {
        let prev_done = self.appearance_manager.done;
        if let Some(new_borns) = self.appearance_manager.update() {
            for enemy in new_borns {
                self.spawn(enemy);
            }
        }
        if !prev_done && self.appearance_manager.done {
            self.done_appearance();
        }
    }

    fn update_formation(&mut self) {
        self.formation.update();
        self.copy_formation_positions();
    }

    fn copy_formation_positions(&mut self) {
        for enemy in self.enemies.iter_mut().flat_map(|x| x).filter(|x| x.state == EnemyState::Formation) {
            let index = enemy.formation_index as usize;
            let pos = self.formation.pos(index & 15, index / 16);
            enemy.pos = pos;
        }
    }

    fn update_enemies(&mut self) {
        for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
            let enemy = enemy_opt.as_mut().unwrap();
            enemy.update(&self.formation);
            if out_of_screen(enemy.pos()) {
                *enemy_opt = None;
            }
        }
    }

    fn update_shots(&mut self) {
        for shot_opt in self.shots.iter_mut().filter(|x| x.is_some()) {
            let shot = shot_opt.as_mut().unwrap();
            shot.update();
            if out_of_screen(shot.pos()) {
                *shot_opt = None;
            }
        }
    }

    fn find_slot(&self) -> Option<usize> {
        self.enemies.iter().position(|x| x.is_none())
    }

    fn spawn_shot(&mut self, pos: &Vec2I, target_pos: &[Option<Vec2I>], speed: u32) {
        if let Some(index) = self.shots.iter().position(|x| x.is_none()) {
            let mut rng = rand::thread_rng();
            let count = target_pos.iter().filter(|x| x.is_some()).count();
            let target_opt: &Option<Vec2I> = target_pos.iter().filter(|x| x.is_some()).nth(rng.gen_range(0, count)).unwrap();
            let target: Vec2I = target_opt.unwrap();
            let d = Vec2I::new(target.x * 256 - pos.x, target.y * 256 - pos.y);
            let distance = ((d.x as f64).powi(2) + (d.y as f64).powi(2)).sqrt();
            let f = (speed as f64) / distance;
            let vel = Vec2I::new(
                ((d.x as f64) * f).round() as i32,
                ((d.y as f64) * f).round() as i32,
            );
            self.shots[index] = Some(EneShot::new(
                *pos, vel,
            ));
        }
    }
}

fn out_of_screen(pos: Vec2I) -> bool {
    pos.x < -16 || pos.x > 224 + 16
        || pos.y < -16 || pos.y > 288 + 16
}

extern crate sdl2;

use rand::Rng;
use sdl2::render::{Texture, WindowCanvas};
use std::mem::MaybeUninit;

use super::enemy::{Enemy, EnemyType, EnemyState};
use super::ene_shot::EneShot;
use super::formation::Formation;
use super::traj::Traj;
use super::traj_command_table::*;
use super::super::util::{CollisionResult, CollBox, Collidable};
use super::super::game::EventQueue;
use super::super::super::util::types::Vec2I;

const MAX_ENEMY_COUNT: usize = 64;
const MAX_SHOT_COUNT: usize = 16;

const ENEMY_COUNT: usize = 40;

lazy_static! {
    static ref ENEMY_TYPE_TABLE: [(EnemyType, usize); ENEMY_COUNT] = {
        let count_table = [4, 8, 8, 10, 10];
        let type_table = [EnemyType::Owl, EnemyType::Butterfly, EnemyType::Butterfly, EnemyType::Bee, EnemyType::Bee];
        let mut buf: [MaybeUninit<(EnemyType, usize)>; ENEMY_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        for i in 0..count_table.len() {
            let enemy_type = type_table[i];
            let count = count_table[i];
            for j in 0..count {
                buf[index] = MaybeUninit::new((enemy_type, i * 16 + (10 - count) / 2 + j));
                index += 1;
            }
        }
        unsafe { std::mem::transmute::<_, [(EnemyType, usize); ENEMY_COUNT]>(buf) }
    };
}

pub struct EnemyManager {
    enemies: [Option<Enemy>; MAX_ENEMY_COUNT],
    shots: [Option<EneShot>; MAX_SHOT_COUNT],
    formation: Formation,
    frame_count: u32,
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
            frame_count: 0,
        };
        mgr.restart();
        mgr
    }

    pub fn restart(&mut self) {
        self.frame_count = 0;

        for slot in self.enemies.iter_mut() {
            *slot = None;
        }
        for slot in self.shots.iter_mut() {
            *slot = None;
        }

        self.formation.restart();

        let angle = 0 * 256;
        let speed = 0;
        for i in 0..ENEMY_TYPE_TABLE.len() {
            let (enemy_type, index) = ENEMY_TYPE_TABLE[i];
            let pos = Vec2I::new(0, 0);
            let mut enemy = Enemy::new(enemy_type, pos, angle, speed);
            enemy.state = EnemyState::Formation;
            enemy.formation_index = index;
            self.enemies[i] = Some(enemy);
        }

        self.copy_formation_positions();
    }

    pub fn update(&mut self, player_pos: &[Option<Vec2I>], event_queue: &mut EventQueue) {
        self.spawn_with_time(player_pos);
        self.update_formation();
        self.update_enemies(event_queue);
        self.update_shots(event_queue);
        self.frame_count += 1;
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

    fn spawn_with_time(&mut self, player_pos: &[Option<Vec2I>]) {
        if self.frame_count % 25 == 0 {
            let mut rng = rand::thread_rng();
            let enemy_type = match rng.gen_range(0, 3) {
                1 => EnemyType::Butterfly,
                2 => EnemyType::Owl,
                _ => EnemyType::Bee,
            };
            let x = rng.gen_range(0 + 8, 224 - 8);
            let angle = rng.gen_range(96, 160) * 256;
            let speed = rng.gen_range(2 * 256, 4 * 256);
            if let Some(index) = self.find_slot() {
                let pos = Vec2I::new(x * 256, -8 * 256);
                let mut enemy = Enemy::new(enemy_type, pos, angle, speed);
                enemy.vangle = rng.gen_range(-512, 512);
                self.enemies[index] = Some(enemy);

                self.spawn_shot(&pos, &player_pos, 3 * 256);
            }
        }


        if (self.frame_count & 255) < 16 / 3 * 4 && (self.frame_count & 255) % (16 / 3) == 0 {
            let flip_x = (self.frame_count & 256) != 0;
            if let Some(index) = self.find_slot() {
                let pos = Vec2I::new(0, 0);
                let enemy_type = EnemyType::Bee;
                let mut enemy = Enemy::new(enemy_type, pos, 0, 0);

                let traj = Traj::new(Some(&COMMAND_TABLE1), Vec2I::new(-8 * 256, 0), flip_x);
                enemy.set_traj(traj);

                self.enemies[index] = Some(enemy);
            }
            if let Some(index) = self.find_slot() {
                let pos = Vec2I::new(0, 0);
                let enemy_type = EnemyType::Butterfly;
                let mut enemy = Enemy::new(enemy_type, pos, 0, 0);

                let traj = Traj::new(Some(&COMMAND_TABLE1), Vec2I::new(8 * 256, 0), flip_x);
                enemy.set_traj(traj);

                self.enemies[index] = Some(enemy);
            }
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

    fn update_enemies(&mut self, event_queue: &mut EventQueue) {
        for enemy_opt in self.enemies.iter_mut().filter(|x| x.is_some()) {
            let enemy = enemy_opt.as_mut().unwrap();
            enemy.update(event_queue);
            if out_of_screen(enemy.pos()) {
                *enemy_opt = None;
            }
        }
    }

    fn update_shots(&mut self, _event_queue: &mut EventQueue) {
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

extern crate sdl2;

use rand::Rng;
use sdl2::render::{Texture, WindowCanvas};
use std::mem::MaybeUninit;

use super::collision::{CollisionResult, CollBox, Collidable};
use super::enemy::Enemy;
use super::game_event_queue::GameEventQueue;
use super::super::util::types::Vec2I;

const MAX_ENEMY_COUNT: usize = 128;

pub struct EnemyManager {
    enemies: [Option<Enemy>; MAX_ENEMY_COUNT],
    frame_count: u32,
}

impl EnemyManager {
    pub fn new() -> EnemyManager {
        let mut enemies: [MaybeUninit<Option<Enemy>>; MAX_ENEMY_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };
        for (_i, element) in enemies.iter_mut().enumerate() {
            *element = MaybeUninit::new(None);
        }
        let enemies = unsafe { std::mem::transmute::<_, [Option<Enemy>; MAX_ENEMY_COUNT]>(enemies) };

        EnemyManager {
            enemies: enemies,
            frame_count: 0,
        }
    }

    pub fn update(&mut self, event_queue: &mut GameEventQueue) {
        self.update_formation();
        self.update_enemies(event_queue);
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &mut Texture) -> Result<(), String> {
        for enemy in self.enemies.iter().flat_map(|x| x) {
            enemy.draw(canvas, texture)?;
        }

        Ok(())
    }

    pub fn check_collision(&mut self, target: &CollBox, power: u32) -> CollisionResult {
        for mut enemy_opt in self.enemies.iter_mut() {
            if let Some(enemy) = &mut enemy_opt {
                if enemy.get_collbox().check_collision(target) {
                    let pos = enemy.pos();
                    let destroyed = enemy.set_damage(power);
                    if destroyed {
                        *enemy_opt = None;
                    }
                    return CollisionResult::Hit(pos, destroyed);
                }
            }
        }
        return CollisionResult::NoHit;
    }

    fn update_formation(&mut self) {
        self.frame_count += 1;
        if self.frame_count % 25 == 0 {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0 + 8, 224 - 8);
            let angle = rng.gen_range(32, 96) * 256;
            let speed = rng.gen_range(2 * 256, 4 * 256);
            self.spawn(Vec2I::new(x, -8), angle, speed);
        }
    }

    fn update_enemies(&mut self, event_queue: &mut GameEventQueue) {
        for enemy_opt in self.enemies.iter_mut() {
            if let Some(enemy) = enemy_opt {
                enemy.update(event_queue);
                if out_of_screen(enemy.pos()) {
                    *enemy_opt = None;
                }
            }
        }
    }

    fn spawn(&mut self, pos: Vec2I, angle: i32, speed: i32) {
        let enemy = Enemy::new(
            pos,
            angle,
            speed,
        );

        if let Some(enemy_opt) = self.enemies.iter_mut().find(|x| x.is_none()) {
            *enemy_opt = Some(enemy);
        }
    }
}

fn out_of_screen(pos: Vec2I) -> bool {
    pos.x < -16 || pos.x > 224 + 16
        || pos.y < -16 || pos.y > 288 + 16
}

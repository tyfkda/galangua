extern crate sdl2;

use sdl2::render::{Texture, WindowCanvas};
use std::mem::MaybeUninit;

use super::collision::{CollisionResult, Collidable};
use super::enemy::Enemy;
use super::game_event_queue::GameEventQueue;
use super::super::util::types::Vec2I;

const MAX_ENEMY_COUNT: usize = 128;

pub struct EnemyManager {
    enemies: [Option<Enemy>; MAX_ENEMY_COUNT],
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
        }
    }

    pub fn update(&mut self, event_queue: &mut GameEventQueue) {
        for enemy in self.enemies.iter_mut().flat_map(|x| x) {
            enemy.update(event_queue);
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &mut Texture) -> Result<(), String> {
        for enemy in self.enemies.iter().flat_map(|x| x) {
            enemy.draw(canvas, texture)?;
        }

        Ok(())
    }

    pub fn spawn(&mut self, pos: Vec2I) {
        let enemy = Enemy::new(
            pos,
        );

        if let Some(enemy_opt) = self.enemies.iter_mut().find(|x| x.is_none()) {
            *enemy_opt = Some(enemy);
        }
    }

    pub fn check_myshot_collision(&mut self, myshot: &Box<&dyn Collidable>) -> CollisionResult {
        for enemy_opt in self.enemies.iter_mut() {
            if let Some(enemy) = &enemy_opt {
                match enemy.collide_with(myshot) {
                    CollisionResult::NoHit => { /* no hit */ },
                    _ => {
                        *enemy_opt = None;
                        return CollisionResult::Destroy;
                    },
                }
            }
        }
        return CollisionResult::NoHit;
    }
}

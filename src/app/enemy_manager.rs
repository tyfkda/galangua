extern crate sdl2;

use rand::Rng;
use sdl2::render::{Texture, WindowCanvas};
use std::mem::MaybeUninit;

use super::collision::{CollisionResult, CollBox, Collidable};
use super::enemy::{Enemy, EnemyType};
use super::game_event_queue::GameEventQueue;
use super::super::util::types::Vec2I;

const MAX_ENEMY_COUNT: usize = 64;

const ENEMY_COUNT: usize = 40;

lazy_static! {
    static ref ENEMY_BASE_POS_TABLE: [Vec2I; ENEMY_COUNT] = {
        let count_table = [4, 8, 8, 10, 10];
        let cx = 224 / 2;
        let by = 32 + 8;
        let w = 16;
        let h = 16;
        let mut buf: [MaybeUninit<Vec2I>; ENEMY_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        for i in 0..count_table.len() {
            let count = count_table[i];
            let y = by + (i as i32) * h;
            for j in 0..count {
                let x = cx - (count - 1) * w / 2 + j * w;
                buf[index] = MaybeUninit::new(Vec2I::new(x, y));
                index += 1;
            }
        }
        unsafe { std::mem::transmute::<_, [Vec2I; ENEMY_COUNT]>(buf) }
    };
    static ref ENEMY_TYPE_TABLE: [EnemyType; ENEMY_COUNT] = {
        let count_table = [4, 8, 8, 10, 10];
        let type_table = [EnemyType::Owl, EnemyType::Butterfly, EnemyType::Butterfly, EnemyType::Bee, EnemyType::Bee];
        let mut buf: [MaybeUninit<EnemyType>; ENEMY_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        for i in 0..count_table.len() {
            let enemy_type = type_table[i];
            let count = count_table[i];
            for _ in 0..count {
                buf[index] = MaybeUninit::new(enemy_type);
                index += 1;
            }
        }
        unsafe { std::mem::transmute::<_, [EnemyType; ENEMY_COUNT]>(buf) }
    };
}

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

        let mut mgr = EnemyManager {
            enemies: enemies,
            frame_count: 0,
        };
        mgr.restart();
        mgr
    }

    pub fn restart(&mut self) {
        for slot in self.enemies.iter_mut() {
            *slot = None;
        }
        self.frame_count = 0;

        let angle = (256 * 3 / 4) * 256;
        let speed = 0;
        for i in 0..ENEMY_BASE_POS_TABLE.len() {
            let pos = ENEMY_BASE_POS_TABLE[i];
            let enemy_type = ENEMY_TYPE_TABLE[i];
            self.spawn(enemy_type, pos, angle, speed, 0);
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
            let enemy_type = match rng.gen_range(0, 3) {
                1 => EnemyType::Butterfly,
                2 => EnemyType::Owl,
                _ => EnemyType::Bee,
            };
            let x = rng.gen_range(0 + 8, 224 - 8);
            let angle = rng.gen_range(32, 96) * 256;
            let speed = rng.gen_range(2 * 256, 4 * 256);
            let vangle = rng.gen_range(-512, 512);
            self.spawn(enemy_type, Vec2I::new(x, -8), angle, speed, vangle);
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

    fn spawn(&mut self, enemy_type: EnemyType, pos: Vec2I, angle: i32, speed: i32, vangle: i32) {
        let mut enemy = Enemy::new(
            enemy_type,
            pos,
            angle,
            speed,
        );
        enemy.vangle = vangle;

        if let Some(enemy_opt) = self.enemies.iter_mut().find(|x| x.is_none()) {
            *enemy_opt = Some(enemy);
        }
    }
}

fn out_of_screen(pos: Vec2I) -> bool {
    pos.x < -16 || pos.x > 224 + 16
        || pos.y < -16 || pos.y > 288 + 16
}

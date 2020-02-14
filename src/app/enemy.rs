extern crate sdl2;

use rand::Rng;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::game_event_queue::GameEventQueue;
use super::super::util::types::Vec2I;

enum EnemyType {
    Bee,
    Butterfly,
    Owl,
}

pub struct Enemy {
    enemy_type: EnemyType,
    life: u32,
    pos: Vec2I,
    vel: Vec2I,
    angle: i32,
}

impl Enemy {
    pub fn new(pos: Vec2I, vel: Vec2I) -> Enemy {
        let mut rng = rand::thread_rng();
        let enemy_type = match rng.gen_range(0, 3) {
            1 => EnemyType::Butterfly,
            2 => EnemyType::Owl,
            _ => EnemyType::Bee,
        };
        let life = match enemy_type {
            EnemyType::Owl => 2,
            _ => 1,
        };
        let angle = rng.gen_range(0, 16);

        Enemy {
            enemy_type,
            life,
            pos,
            vel,
            angle,
        }
    }

    pub fn pos(&self) -> Vec2I {
        self.pos
    }

    pub fn update(&mut self, _event_queue: &mut GameEventQueue) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
        let src_u = match self.enemy_type {
            EnemyType::Owl => if self.life <= 1 { 32 } else { 0 },
            _ => 0,
        };
        let src_v = match self.enemy_type {
            EnemyType::Bee => 16,
            EnemyType::Butterfly => 32,
            EnemyType::Owl => 48,
        };

        let angle: f64 = ((self.angle & 15) as f64) * (360.0 / 16.0);

        canvas.copy_ex(&texture,
                       Some(Rect::new(src_u, src_v, 16, 16)),
                       Some(Rect::new((self.pos.x - 8) * 2, (self.pos.y - 8) * 2, 16 * 2, 16 * 2)),
                       angle,
                       None,
                       false, false)?;

        Ok(())
    }

    pub fn set_damage(&mut self, power: u32) -> bool {
        if self.life > power {
            self.life -= power;
            false
        } else {
            self.life = 0;
            true
        }
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: Vec2I::new(self.pos.x - 8, self.pos.y - 8),
            size: Vec2I::new(16, 16),
        }
    }
}

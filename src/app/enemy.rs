extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::game_event_queue::GameEventQueue;
use super::super::util::types::Vec2I;
use super::super::util::math::{SIN_TABLE, COS_TABLE};

#[derive(Copy, Clone)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
}

pub struct Enemy {
    enemy_type: EnemyType,
    life: u32,
    pub pos: Vec2I,
    pub angle: i32,
    pub speed: i32,
    pub vangle: i32,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, pos: Vec2I, angle: i32, speed: i32) -> Enemy {
        let life = match enemy_type {
            EnemyType::Owl => 2,
            _ => 1,
        };

        Enemy {
            enemy_type,
            life,
            pos: Vec2I::new(pos.x * 256, pos.y * 256),
            angle,
            speed,
            vangle: 0,
        }
    }

    pub fn pos(&self) -> Vec2I {
        Vec2I::new((self.pos.x + 128) / 256, (self.pos.y + 128) / 256)
    }

    pub fn update(&mut self, _event_queue: &mut GameEventQueue) {
        self.angle += self.vangle;
        let (vx, vy) = calc_velocity(self.angle, self.speed);

        self.pos.x += vx;
        self.pos.y += vy;
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

        let angle = calc_display_angle(self.angle);
        let pos = self.pos();
        canvas.copy_ex(&texture,
                       Some(Rect::new(src_u, src_v, 16, 16)),
                       Some(Rect::new((pos.x - 8) * 2, (pos.y - 8) * 2, 16 * 2, 16 * 2)),
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
        let pos = self.pos();
        CollBox {
            top_left: Vec2I::new(pos.x - 8, pos.y - 8),
            size: Vec2I::new(16, 16),
        }
    }
}

fn calc_velocity(angle: i32, speed: i32) -> (i32, i32) {
    let a: usize = (((angle + 128) / 256) & 255) as usize;
    let cs = COS_TABLE[a];
    let sn = SIN_TABLE[a];
    (cs * speed / 256, sn * speed / 256)
}

fn calc_display_angle(angle: i32) -> f64 {
    //let angle: f64 = (self.angle as f64) * (360.0 / (256.0 * 256.0)) + 90.0;

    // Quantize.
    let div = 16;
    let angle = (((angle + 256 * 256 / div / 2) / (256 * 256 / div)) & (div - 1)) + div / 4;
    let angle: f64 = (angle as f64) * (360.0 / (div as f64));

    angle
}

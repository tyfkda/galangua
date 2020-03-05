extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::formation::Formation;
use super::traj::Traj;
use super::super::util::{CollBox, Collidable};
use super::super::super::util::math::{calc_velocity, clamp, diff_angle, round_up, ANGLE, ONE};
use super::super::super::util::types::Vec2I;

#[derive(Copy, Clone)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EnemyState {
    Flying,
    Trajectory,
    MoveToFormation,
    Formation,
}

pub struct Enemy {
    pub state: EnemyState,
    pub pos: Vec2I,
    pub angle: i32,
    pub speed: i32,
    pub vangle: i32,
    pub formation_index: usize,

    enemy_type: EnemyType,
    life: u32,
    traj: Option<Traj>,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, pos: Vec2I, angle: i32, speed: i32) -> Enemy {
        let life = match enemy_type {
            EnemyType::Owl => 2,
            _ => 1,
        };

        Enemy {
            enemy_type,
            state: EnemyState::Flying,
            life,
            pos,
            angle,
            speed,
            vangle: 0,
            formation_index: 255,  // Dummy
            traj: None,
        }
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn update(&mut self, formation: &Formation) {
        if self.state == EnemyState::Formation {
            return;
        }

        match self.state {
            EnemyState::Flying => {}
            EnemyState::Trajectory => {
                let cont = self.update_traj();
                if !cont {
                    self.state = EnemyState::MoveToFormation;
                }
            }
            EnemyState::MoveToFormation => {
                let ix = self.formation_index & 15;
                let iy = self.formation_index / 16;
                let target = formation.pos(ix, iy);
                let dpos = Vec2I::new(target.x - self.pos.x, target.y - self.pos.y);

                let distance = ((dpos.x as f64).powi(2) + (dpos.y as f64).powi(2)).sqrt();
                if distance > self.speed as f64 {
                    let dlimit = 3 * ONE;
                    let target_angle_rad = (dpos.x as f64).atan2(-dpos.y as f64);
                    let target_angle = ((target_angle_rad * (((ANGLE * ONE) as f64) / (2.0 * std::f64::consts::PI)) + 0.5).floor() as i32) & (ANGLE * ONE - 1);
                    let d = diff_angle(target_angle, self.angle);
                    self.angle += clamp(d, -dlimit, dlimit);
                    self.vangle = 0;
                } else {
                    self.pos = target;
                    self.speed = 0;
                    self.angle = 0;
                    self.state = EnemyState::Formation;
                }
            }
            _ => {}
        }

        self.pos += calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;
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

    pub fn set_traj(&mut self, traj: Traj) {
        self.traj = Some(traj);
        self.state = EnemyState::Trajectory;
    }

    fn update_traj(&mut self) -> bool {
        if let Some(traj) = &mut self.traj {
            let cont = traj.update();

            self.pos = traj.pos();
            self.angle = traj.angle();
            self.speed = traj.speed;
            self.vangle = traj.vangle;

            cont
        } else {
            false
        }
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: self.pos() - Vec2I::new(8, 8),
            size: Vec2I::new(12, 12),
        }
    }
}

fn calc_display_angle(angle: i32) -> f64 {
    // Quantize.
    let div = 16;
    let angle = (angle + ANGLE * ONE / div / 2) & (ANGLE * ONE - (ANGLE * ONE / div));
    let angle: f64 = (angle as f64) * (360.0 / ((ANGLE * ONE) as f64));

    angle
}

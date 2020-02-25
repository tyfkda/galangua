extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

use super::collision::{CollBox, Collidable};
use super::enemy_command::EnemyCommand;
use super::event_queue::EventQueue;
use super::super::util::types::Vec2I;
use super::super::util::math::{SIN_TABLE, COS_TABLE};

#[derive(Copy, Clone)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EnemyState {
    Flying,
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
    command_table: Option<&'static [EnemyCommand]>,
    command_delay: u32,
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
            command_table: None,
            command_delay: 0,
        }
    }

    pub fn pos(&self) -> Vec2I {
        Vec2I::new((self.pos.x + 128) / 256, (self.pos.y + 128) / 256)
    }

    pub fn update(&mut self, _event_queue: &mut EventQueue) {
        if self.state == EnemyState::Flying {
            self.handle_command();

            let (vx, vy) = calc_velocity(self.angle + self.vangle / 2, self.speed);
            self.angle += self.vangle;

            self.pos.x += vx;
            self.pos.y += vy;
        }
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

    pub fn set_command_table(&mut self, command_table: &'static [EnemyCommand]) {
        self.command_table = Some(command_table);
    }

    fn handle_command(&mut self) {
        if let Some(command_table) = self.command_table {
            if self.command_delay > 0 {
                self.command_delay -= 1;
                return;
            }

            let mut i = 0;
            while i < command_table.len() {
                let command = command_table[i];
                i += 1;
                match command {
                    EnemyCommand::Pos(x, y) => {
                        self.pos = Vec2I::new(x, y);
                    },
                    EnemyCommand::Speed(speed) => {
                        self.speed = speed;
                    },
                    EnemyCommand::Angle(angle) => {
                        self.angle = angle;
                    },
                    EnemyCommand::VAngle(vangle) => {
                        self.vangle = vangle;
                    },
                    EnemyCommand::Delay(delay) => {
                        self.command_delay = delay;
                        break;
                    },
                    EnemyCommand::DestAngle(dest_angle, radius) => {
                        let distance = 2.0 * std::f64::consts::PI * (radius as f64) / 256.0;  // 半径radiusの円周
                        let frame = distance * 256.0 / (self.speed as f64);  // 速度speedで動いたときにかかるフレーム数[frame]
                        let dangle = (2.0 * std::f64::consts::PI) / frame;  // １フレームあたりに変化させるべき角度[rad]

                        let vangle = dangle * (256.0 * 256.0 / (2.0 * std::f64::consts::PI));
                        if dest_angle > self.angle {
                            self.vangle = vangle.round() as i32;
                            self.command_delay = (((dest_angle - self.angle) as f64) / vangle).round() as u32;
                        } else {
                            self.vangle = -vangle.round() as i32;
                            self.command_delay = (((self.angle - dest_angle) as f64) / vangle).round() as u32;
                        }
                        break;
                    },
                }
            }

            if i < command_table.len() {
                self.command_table = Some(&command_table[i .. command_table.len()]);
            } else {
                self.command_table = None;
            }
        }
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> CollBox {
        let pos = self.pos();
        CollBox {
            top_left: Vec2I::new(pos.x - 8, pos.y - 8),
            size: Vec2I::new(12, 12),
        }
    }
}

fn calc_velocity(angle: i32, speed: i32) -> (i32, i32) {
    let a: usize = (((angle + 128) & (255 * 256)) / 256) as usize;
    let cs = COS_TABLE[a];
    let sn = SIN_TABLE[a];
    (sn * speed / 256, -cs * speed / 256)
}

fn calc_display_angle(angle: i32) -> f64 {
    //let angle: f64 = (self.angle as f64) * (360.0 / (256.0 * 256.0)) + 90.0;

    // Quantize.
    let div = 16;
    let angle = (angle + 256 * 256 / div / 2) & (256 * 256 - (256 * 256 / div));
    let angle: f64 = (angle as f64) * (360.0 / (256.0 * 256.0));

    angle
}

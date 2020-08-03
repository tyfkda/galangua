use super::traj_command::TrajCommand;

use crate::app::consts::*;
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::{calc_velocity, ANGLE, COS_TABLE, ONE, SIN_TABLE};

#[cfg(debug_assertions)]
use crate::app::util::unsafe_util::extend_lifetime;

// Trajectory
#[derive(Clone, Debug)]
pub struct Traj {
    pos: Vec2I,
    angle: i32,
    pub speed: i32,
    pub vangle: i32,
    offset: Vec2I,
    flip_x: bool,

    command_table: &'static [TrajCommand],
    delay: u32,

    command_table_vec: Option<Vec<TrajCommand>>,
}

impl Traj {
    pub fn new(command_table: &'static [TrajCommand], offset: &Vec2I, flip_x: bool) -> Self {
        let offset = if flip_x { Vec2I::new(-offset.x, offset.y) } else { *offset };
        Self {
            pos: ZERO_VEC,
            angle: 0,
            speed: 0,
            vangle: 0,
            offset,
            flip_x,

            command_table: command_table,
            delay: 0,

            command_table_vec: None,
        }
    }

    #[cfg(debug_assertions)]
    pub fn new_with_vec(command_table_vec: Vec<TrajCommand>, offset: &Vec2I, flip_x: bool) -> Self {
        // command_table is owned by vec, so it lives as long as self and not worry about that.
        let command_table = unsafe { extend_lifetime(&command_table_vec) };

        let mut me = Self::new(command_table, offset, flip_x);
        me.command_table_vec = Some(command_table_vec);
        me
    }

    pub fn pos(&self) -> Vec2I {
        let a: usize = (((self.angle + ONE / 2) & ((ANGLE - 1) * ONE)) / ONE) as usize;
        let cs = COS_TABLE[a];
        let sn = SIN_TABLE[a];
        let x = self.pos.x + (cs * self.offset.x + sn * self.offset.y) / ONE;
        let y = self.pos.y + (sn * self.offset.x - cs * self.offset.y) / ONE;
        Vec2I::new(x, y)
    }

    pub fn set_pos(&mut self, pos: &Vec2I) {
        self.pos = pos.clone();
    }

    pub fn angle(&self) -> i32 {
        self.angle
    }

    pub fn update(&mut self) -> bool {
        self.handle_command();

        self.pos += calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;

        !self.command_table.is_empty() || self.delay > 0
    }

    fn handle_command(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        }

        if !self.command_table.is_empty() {
            let mut i = 0;
            while i < self.command_table.len() {
                let command = &self.command_table[i];
                i += 1;
                if !self.handle_one_command(command) {
                    break;
                }
            }

            self.command_table = &self.command_table[i..];
        }
    }

    fn handle_one_command(&mut self, command: &TrajCommand) -> bool {
        match *command {
            TrajCommand::Pos(mut x, y) => {
                if self.flip_x {
                    x = WIDTH * ONE - x;
                }
                self.pos = Vec2I::new(x, y);
            }
            TrajCommand::Speed(speed) => {
                self.speed = speed;
            }
            TrajCommand::Angle(mut angle) => {
                if self.flip_x {
                    angle = -angle;
                }
                self.angle = angle;
            }
            TrajCommand::VAngle(mut vangle) => {
                if self.flip_x {
                    vangle = -vangle;
                }
                self.vangle = vangle;
            }
            TrajCommand::Delay(delay) => {
                self.delay = delay;
                return false;
            }
            TrajCommand::DestAngle(mut dest_angle, radius) => {
                if self.flip_x {
                    dest_angle = -dest_angle;
                }
                let distance = 2.0 * std::f64::consts::PI * (radius as f64) / (ONE as f64);  // Circumference of radius [dot].
                let frame = distance * (ONE as f64) / (self.speed as f64);  // Frame count according to speed [frame].
                let dangle = (2.0 * std::f64::consts::PI) / frame;  // Angle which should be modified in one frame [rad].
                let vangle = dangle * (((ANGLE * ONE) as f64) / (2.0 * std::f64::consts::PI));  // [ANGLE * ONE]
                if dest_angle > self.angle {
                    self.vangle = vangle.round() as i32;
                    self.delay = (((dest_angle - self.angle) as f64) / vangle).round() as u32;
                } else {
                    self.vangle = -vangle.round() as i32;
                    self.delay = (((self.angle - dest_angle) as f64) / vangle).round() as u32;
                }
                return false;
            }
        }
        true
    }
}

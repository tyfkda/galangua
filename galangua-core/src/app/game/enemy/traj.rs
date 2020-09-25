use super::traj_command::TrajCommand;
use super::traj_command::TrajCommand::*;
use super::{Accessor, FormationIndex};

use crate::app::consts::*;
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::util::math::{calc_velocity, ANGLE, COS_TABLE, ONE, SIN_TABLE};

#[cfg(debug_assertions)]
use crate::app::util::unsafe_util::extend_lifetime;

// Trajectory
pub struct Traj {
    pos: Vec2I,
    pub angle: i32,
    pub speed: i32,
    pub vangle: i32,
    offset: Vec2I,
    flip_x: bool,
    fi: FormationIndex,
    pub shot_enable: bool,

    command_table: &'static [TrajCommand],
    delay: u32,
    wait_pred: Option<Box<dyn Fn(&Vec2I) -> bool>>,
    shot: Option<u32>,

    #[cfg(debug_assertions)]
    command_table_vec: Option<Vec<TrajCommand>>,
}

impl Traj {
    pub fn new(
        command_table: &'static [TrajCommand], offset: &Vec2I, flip_x: bool,
        fi: FormationIndex,
    ) -> Self {
        let offset = if flip_x { Vec2I::new(-offset.x, offset.y) } else { *offset };
        Self {
            pos: ZERO_VEC,
            angle: 0,
            speed: 0,
            vangle: 0,
            offset,
            flip_x,
            fi,
            shot_enable: true,

            command_table,
            delay: 0,
            wait_pred: None,
            shot: None,

            #[cfg(debug_assertions)]
            command_table_vec: None,
        }
    }

    #[cfg(debug_assertions)]
    pub fn new_with_vec(
        command_table_vec: Vec<TrajCommand>, offset: &Vec2I, flip_x: bool,
        fi: FormationIndex,
    ) -> Self {
        // command_table is owned by vec, so it lives as long as self and not worry about that.
        let command_table = unsafe { extend_lifetime(&command_table_vec) };

        let mut me = Self::new(command_table, offset, flip_x, fi);
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

    pub fn is_shot(&mut self) -> Option<u32> {
        self.shot.take()
    }

    pub fn update(&mut self, accessor: &dyn Accessor) -> bool {
        self.handle_command(accessor);

        self.pos += &calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;

        !self.command_table.is_empty() || self.delay > 0
    }

    fn handle_command(&mut self, accessor: &dyn Accessor) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        }
        if let Some(wait_pred) = &self.wait_pred {
            if !(wait_pred)(&self.pos) {
                return;
            }
        }

        if !self.command_table.is_empty() {
            let mut i = 0;
            while i < self.command_table.len() {
                let command = &self.command_table[i];
                i += 1;
                if !self.handle_one_command(command, accessor) {
                    break;
                }
            }

            self.command_table = &self.command_table[i..];
        }
    }

    fn handle_one_command(&mut self, command: &TrajCommand, accessor: &dyn Accessor) -> bool {
        match *command {
            Pos(mut x, y) => {
                if self.flip_x {
                    x = WIDTH * ONE - x;
                }
                self.pos = Vec2I::new(x, y);
            }
            Speed(speed) => {
                self.speed = speed;
            }
            Angle(mut angle) => {
                if self.flip_x {
                    angle = -angle;
                }
                self.angle = angle;
            }
            VAngle(mut vangle) => {
                if self.flip_x {
                    vangle = -vangle;
                }
                self.vangle = vangle;
            }
            Delay(delay) => {
                self.delay = delay;
                return false;
            }
            Accelerate => {
                const MAX_STAGE: i32 = 64;
                const MAX_SPEED: i32 = 5 * ONE;
                let stage_no = accessor.get_stage_no();
                self.speed += (MAX_SPEED - self.speed) * std::cmp::min(stage_no as i32, MAX_STAGE) / MAX_STAGE;
            }
            DestAngle(mut dest_angle, radius) => {
                if self.flip_x {
                    dest_angle = -dest_angle;
                }
                let distance = 2.0 * std::f32::consts::PI * (radius as f32) / (ONE as f32);  // Circumference of radius [dot].
                let frame = distance * (ONE as f32) / (self.speed as f32);  // Frame count according to speed [frame].
                let dangle = (2.0 * std::f32::consts::PI) / frame;  // Angle which should be modified in one frame [rad].
                let vangle = dangle * (((ANGLE * ONE) as f32) / (2.0 * std::f32::consts::PI));  // [ANGLE * ONE]
                if dest_angle > self.angle {
                    self.vangle = vangle.round() as i32;
                    self.delay = (((dest_angle - self.angle) as f32) / vangle).round() as u32;
                } else {
                    self.vangle = -vangle.round() as i32;
                    self.delay = (((self.angle - dest_angle) as f32) / vangle).round() as u32;
                }
                return false;
            }
            WaitYG(value) => {
                self.wait_pred = Some(Box::new(move |&pos| pos.y >= value));
                return false;
            }
            AddPos(mut x, y) => {
                if self.flip_x {
                    x = -x;
                }
                self.pos.x += x;
                self.pos.y += y;
            }
            CopyFormationX => {
                let formation_pos = accessor.get_formation_pos(&self.fi);
                self.pos.x = formation_pos.x;
            }
            Shot(delay) => {
                if self.shot_enable {
                    self.shot = Some(delay);
                }
            }
        }
        true
    }
}

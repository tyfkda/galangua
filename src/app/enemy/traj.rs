use super::traj_command::TrajCommand;
use super::super::super::util::types::Vec2I;
use super::super::super::util::math::{SIN_TABLE, COS_TABLE, calc_velocity};

// Trajectory
pub struct Traj {
    pos: Vec2I,
    angle: i32,
    pub speed: i32,
    pub vangle: i32,
    offset: Vec2I,
    flip_x: bool,

    command_table: Option<&'static [TrajCommand]>,
    command_delay: u32,
}

impl Traj {
    pub fn new(command_table: Option<&'static [TrajCommand]>, offset: Vec2I, flip_x: bool) -> Traj {
        Traj {
            pos: Vec2I::new(0, 0),
            angle: 0,
            speed: 0,
            vangle: 0,
            offset,
            flip_x,

            command_table,
            command_delay: 0,
        }
    }

    pub fn pos(&self) -> Vec2I {
        let a: usize = (((self.angle + 128) & (255 * 256)) / 256) as usize;
        let cs = COS_TABLE[a];
        let sn = SIN_TABLE[a];
        let mut x = self.pos.x + (cs * self.offset.x + sn * self.offset.y) / 256;
        let y = self.pos.y + (sn * self.offset.x - cs * self.offset.y) / 256;
        if self.flip_x {
            x = 224 * 256 - x;
        }
        Vec2I::new(x, y)
    }

    pub fn angle(&self) -> i32 {
        if self.flip_x {
            -self.angle & (256 * 256 - 1)
        } else {
            self.angle
        }
    }

    pub fn update(&mut self) -> bool {
        self.handle_command();

        let (vx, vy) = calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;

        self.pos.x += vx;
        self.pos.y += vy;

        self.command_table.is_some()
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
                    TrajCommand::Pos(x, y) => {
                        self.pos = Vec2I::new(x, y);
                    },
                    TrajCommand::Speed(speed) => {
                        self.speed = speed;
                    },
                    TrajCommand::Angle(angle) => {
                        self.angle = angle;
                    },
                    TrajCommand::VAngle(vangle) => {
                        self.vangle = vangle;
                    },
                    TrajCommand::Delay(delay) => {
                        self.command_delay = delay;
                        break;
                    },
                    TrajCommand::DestAngle(dest_angle, radius) => {
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

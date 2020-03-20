use crate::app::consts::*;
use crate::app::enemy::formation::Formation;
use crate::app::enemy::tractor_beam::TractorBeam;
use crate::app::enemy::traj::Traj;
use crate::app::game::EventQueue;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{calc_velocity, clamp, diff_angle, round_up, ANGLE, ONE};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyState {
    Flying,
    Trajectory,
    MoveToFormation,
    Formation,
    Attack,
}

#[derive(Debug)]
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
    attack_step: i32,
    count: i32,
    target_pos: Vec2I,
    tractor_beam: Option<TractorBeam>,
    capturing_player: bool,
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
            attack_step: 0,
            count: 0,
            target_pos: Vec2I::new(0, 0),
            tractor_beam: None,
            capturing_player: false,
        }
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn raw_pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn update(&mut self, formation: &Formation, player_pos: &Vec2I, event_queue: &mut EventQueue) {
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
                    const DLIMIT: i32 = 4 * ONE;
                    let target_angle_rad = (dpos.x as f64).atan2(-dpos.y as f64);
                    let target_angle = ((target_angle_rad * (((ANGLE * ONE) as f64) / (2.0 * std::f64::consts::PI)) + 0.5).floor() as i32) & (ANGLE * ONE - 1);
                    let d = diff_angle(target_angle, self.angle);
                    self.angle += clamp(d, -DLIMIT, DLIMIT);
                    self.vangle = 0;
                } else {
                    self.pos = target;
                    self.speed = 0;
                    self.angle = 0;
                    self.state = EnemyState::Formation;
                }
            }
            EnemyState::Attack => {
                self.update_attack(player_pos, event_queue);
            }
            _ => {}
        }

        self.pos += calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;

        if let Some(tractor_beam) = &mut self.tractor_beam {
            tractor_beam.update();
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        let sprite = match self.enemy_type {
            EnemyType::Bee => "bee",
            EnemyType::Butterfly => "butterfly",
            EnemyType::Owl => {
                if self.life <= 1 { "owl2" } else { "owl1" }
            }
        };

        let angle = calc_display_angle(self.angle);
        let pos = self.pos();
        renderer.draw_sprite_rot(sprite, pos + Vec2I::new(-8, -8), angle, None)?;

        if let Some(tractor_beam) = &self.tractor_beam {
            tractor_beam.draw(renderer)?;
        }
        if self.capturing_player {
            renderer.draw_sprite("captured", pos + Vec2I::new(-8, -8 - 16))?;
        }

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

    pub fn set_attack(&mut self) {
        self.state = EnemyState::Attack;
        self.attack_step = 0;
        self.count = 0;
    }

    fn update_attack(&mut self, player_pos: &Vec2I, event_queue: &mut EventQueue) {
        match self.enemy_type {
            EnemyType::Bee => { self.update_attack_bee(event_queue); }
            EnemyType::Butterfly => { self.update_attack_butterfly(event_queue); }
            EnemyType::Owl => { self.update_attack_owl(player_pos, event_queue); }
        }
    }

    fn update_attack_bee(&mut self, event_queue: &mut EventQueue) {
        match self.attack_step {
            0 => {
                self.speed = 1 * ONE;
                self.angle = 0;
                if (self.formation_index & 15) < 5 {
                    self.vangle = -4 * ONE;
                } else {
                    self.vangle = 4 * ONE;
                }
                self.attack_step += 1;
                self.count = 0;

                event_queue.spawn_ene_shot(self.pos, 2 * ONE);
            }
            1 => {
                if (self.vangle < 0 && self.angle <= -160 * ONE) || (self.vangle > 0 && self.angle >= 160 * ONE) {
                    self.vangle = 0;
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            2 => {
                self.count += 1;
                if self.count >= 10 {
                    if (self.formation_index & 15) < 5 {
                        self.vangle = 1 * ONE / 4;
                    } else {
                        self.vangle = -1 * ONE / 4;
                    }
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            3 => {
                if (self.vangle > 0 && self.angle >= -ANGLE / 2 * ONE) || (self.vangle < 0 && self.angle <= ANGLE / 2 * ONE) {
                    self.vangle = 0;
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            4 => {
                if self.pos.y >= (HEIGHT + 16) * ONE {
                    // TODO: Warp to the top of the screen.
                    self.state = EnemyState::Formation;
                    self.speed = 0;
                    self.angle = 0;
                    self.vangle = 0;
                }
            }
            _ => {}
        }
    }

    fn update_attack_butterfly(&mut self, event_queue: &mut EventQueue) {
        self.update_attack_bee(event_queue);
    }

    fn update_attack_owl(&mut self, player_pos: &Vec2I, event_queue: &mut EventQueue) {
        const DLIMIT: i32 = 4 * ONE;
        match self.attack_step {
            0 => {
                self.speed = 3 * ONE / 2;
                self.angle = 0;
                if (self.formation_index & 15) < 5 {
                    self.vangle = -DLIMIT;
                } else {
                    self.vangle = DLIMIT;
                }

                self.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

                self.attack_step += 1;
                self.count = 0;
            }
            1 => {
                let dpos = self.target_pos - self.pos;
                let target_angle_rad = (dpos.x as f64).atan2(-dpos.y as f64);
                let target_angle = ((target_angle_rad * (((ANGLE * ONE) as f64) / (2.0 * std::f64::consts::PI)) + 0.5).floor() as i32) & (ANGLE * ONE - 1);
                let mut d = diff_angle(target_angle, self.angle);
                if self.vangle > 0 && d < 0 {
                    d += ANGLE * ONE;
                } else if self.vangle < 0 && d > 0 {
                    d -= ANGLE * ONE;
                }
                if d >= -DLIMIT && d < DLIMIT {
                    self.angle = target_angle;
                    self.vangle = 0;
                }

                if self.pos.y >= self.target_pos.y {
                    self.pos.y = self.target_pos.y;
                    self.speed = 0;
                    self.angle = ANGLE / 2 * ONE;
                    self.vangle = 0;

                    self.tractor_beam = Some(TractorBeam::new(self.pos + Vec2I::new(0, 8 * ONE)));

                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            2 => {
                if let Some(tractor_beam) = &mut self.tractor_beam {
                    if tractor_beam.closed() {
                        self.tractor_beam = None;
                        self.speed = 3 * ONE / 2;
                        self.attack_step += 1;
                        self.count = 0;
                    } else if tractor_beam.can_capture(player_pos) {
                        event_queue.capture_player(self.pos + Vec2I::new(0, 16 * ONE));
                        tractor_beam.start_capture();
                        self.attack_step = 100;
                        self.count = 0;
                    }
                }
            }
            3 => {
                if self.pos.y >= (HEIGHT + 16) * ONE {
                    // TODO: Warp to the top of the screen.
                    self.state = EnemyState::Formation;
                    self.speed = 0;
                    self.angle = 0;
                    self.vangle = 0;
                }
            }
            // Capture sequence
            100 => {
                self.count += 1;
                if self.count >= 80 {  // TODO: Synchronize with player capturing duration.
                    self.tractor_beam.as_mut().unwrap().close_capture();
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            101 => {
                if let Some(tractor_beam) = &self.tractor_beam {
                    if tractor_beam.closed() {
                        // TODO: Turn and back to the formation.
                        self.tractor_beam = None;
                        self.capturing_player = true;
                        event_queue.capture_player_completed();

                        self.speed = 3 * ONE / 2;
                        self.attack_step = 3;
                        self.count = 0;
                    }
                }
            }
            _ => {}
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

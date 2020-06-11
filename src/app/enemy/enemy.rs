use crate::app::consts::*;
use crate::app::enemy::tractor_beam::TractorBeam;
use crate::app::enemy::traj::Traj;
use crate::app::enemy::{Accessor, FormationIndex};
use crate::app::game::{EventQueue, EventType};
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{calc_velocity, clamp, diff_angle, quantize_angle, round_up, ANGLE, ONE};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    Bee,
    Butterfly,
    Owl,
    CapturedFighter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyState {
    None,
    Trajectory,
    MoveToFormation,
    Formation,
    Attack,
    Troop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureState {
    None,
    BeamTracting,
    BeamClosing,
    Capturing,
}

const MAX_TROOPS: usize = 3;

#[derive(Debug)]
pub struct Enemy {
    pub state: EnemyState,
    pos: Vec2I,
    angle: i32,
    speed: i32,
    vangle: i32,
    pub formation_index: FormationIndex,

    pub(super) enemy_type: EnemyType,
    life: u32,
    traj: Option<Traj>,
    attack_type: i32,
    attack_step: i32,
    count: i32,
    target_pos: Vec2I,
    tractor_beam: Option<TractorBeam>,
    capture_state: CaptureState,
    troops: Option<[Option<FormationIndex>; MAX_TROOPS]>,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, pos: &Vec2I, angle: i32, speed: i32) -> Self {
        let life = match enemy_type {
            EnemyType::Owl => 2,
            _ => 1,
        };

        Self {
            enemy_type,
            state: EnemyState::None,
            life,
            pos: *pos,
            angle,
            speed,
            vangle: 0,
            formation_index: FormationIndex(255, 255),  // Dummy
            traj: None,
            attack_type: 0,
            attack_step: 0,
            count: 0,
            target_pos: Vec2I::new(0, 0),
            tractor_beam: None,
            capture_state: CaptureState::None,
            troops: None,
        }
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn raw_pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn capture_state(&self) -> CaptureState {
        self.capture_state
    }

    pub fn captured_fighter_index(&self) -> Option<FormationIndex> {
        if self.capture_state == CaptureState::Capturing && self.troops.is_some() {
            let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
            if self.troops.unwrap().iter().flat_map(|x| x)
                .find(|index| **index == fi).is_some()
            {
                return Some(fi);
            }
        }
        None
    }

    pub fn update<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        let prev_pos = self.pos;

        match self.state {
            EnemyState::None | EnemyState::Troop => {}
            EnemyState::Formation => {
                self.pos = accessor.get_formation_pos(&self.formation_index);
            }
            EnemyState::Trajectory => {
                let cont = self.update_traj();
                if !cont {
                    self.state = EnemyState::MoveToFormation;
                }
            }
            EnemyState::MoveToFormation => {
                let target = accessor.get_formation_pos(&self.formation_index);
                let dpos = Vec2I::new(target.x - self.pos.x, target.y - self.pos.y);

                let distance = ((dpos.x as f64).powi(2) + (dpos.y as f64).powi(2)).sqrt();
                if distance > self.speed as f64 {
                    const DLIMIT: i32 = 5 * ONE;
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
                self.update_attack(accessor, event_queue);
            }
        }

        self.pos += calc_velocity(self.angle + self.vangle / 2, self.speed);
        self.angle += self.vangle;

        self.update_troops(&(&self.pos - &prev_pos), self.angle, accessor);

        if let Some(tractor_beam) = &mut self.tractor_beam {
            tractor_beam.update();
        }
    }

    fn update_troops<T: Accessor>(&mut self, add: &Vec2I, angle: i32, accessor: &mut T) {
        if let Some(troops) = &mut self.troops {
            for troop_opt in troops.iter_mut() {
                if let Some(formation_index) = troop_opt {
                    if let Some(enemy) = accessor.get_enemy_at_mut(formation_index) {
                        enemy.update_troop(add, angle);
                    } else {
                        //*troop_opt = None;
                    }
                }
            }
        }
    }

    fn update_troop(&mut self, add: &Vec2I, angle: i32) -> bool {
        self.pos += *add;
        self.angle = angle;
        true
    }

    fn release_troops<T: Accessor>(&mut self, accessor: &mut T) {
        if let Some(troops) = &mut self.troops {
            troops.iter().flat_map(|x| x)
                .for_each(|index| {
                    if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                        enemy.set_to_formation();
                    }
                });
            self.troops = None;
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        let sprite = match self.enemy_type {
            EnemyType::Bee => "gopher",
            EnemyType::Butterfly => "dman",
            EnemyType::Owl => {
                if self.life <= 1 { "cpp2" } else { "cpp1" }
            }
            EnemyType::CapturedFighter => "rustacean_captured",
        };

        let angle = quantize_angle(self.angle, 16);
        let pos = self.pos();
        renderer.draw_sprite_rot(sprite, &(&pos + &Vec2I::new(-8, -8)), angle, None)?;

        if let Some(tractor_beam) = &self.tractor_beam {
            tractor_beam.draw(renderer)?;
        }

        Ok(())
    }

    pub fn set_damage(&mut self, power: u32) -> (bool, u32) {
        if self.life > power {
            self.life -= power;
            (false, 0)
        } else {
            self.life = 0;
            let point = match self.enemy_type {
                EnemyType::Bee => {
                    if self.state == EnemyState::Formation { 50 } else { 100 }
                }
                EnemyType::Butterfly => {
                    if self.state == EnemyState::Formation { 80 } else { 160 }
                }
                EnemyType::Owl => {
                    if self.state == EnemyState::Formation {
                        150
                    } else {
                        let count = if let Some(troops) = &mut self.troops {
                            troops.iter().flat_map(|x| x).count()
                        } else {
                            0
                        };
                        match count {
                            0 => 400,
                            1 => 800,
                            2 | _ => 1600,
                        }
                    }
                }
                EnemyType::CapturedFighter => { 1000 }
            };
            (true, point)
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

    pub fn set_attack<T: Accessor>(&mut self, capture_attack: bool, accessor: &mut T) {
        self.state = EnemyState::Attack;
        self.attack_type = if capture_attack { 1 } else { 0 };
        self.attack_step = 0;
        self.count = 0;

        self.troops = None;
        if !capture_attack && self.enemy_type == EnemyType::Owl {
            self.choose_troops(accessor);
        }
    }

    fn choose_troops<T: Accessor>(&mut self, accessor: &mut T) {
        let base = &self.formation_index;
        let indices = [
            FormationIndex(base.0 - 1, base.1 + 1),
            FormationIndex(base.0 + 1, base.1 + 1),
            FormationIndex(base.0, base.1 - 1),
        ];
        for index in indices.iter() {
            if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                if enemy.state == EnemyState::Formation {
                    self.add_troop(*index);
                }
            }
        }
        if let Some(troops) = self.troops {
            troops.iter().flat_map(|x| x)
                .for_each(|index| {
                    if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                        enemy.set_to_troop();
                    }
                });
        }
    }

    fn add_troop(&mut self, formation_index: FormationIndex) -> bool {
        if self.troops.is_none() {
            self.troops = Some(Default::default());
        }

        if let Some(slot) = self.troops.as_mut().unwrap().iter_mut().find(|x| x.is_none()) {
            *slot = Some(formation_index);
            true
        } else {
            false
        }
    }

    fn set_to_troop(&mut self) {
        self.state = EnemyState::Troop;
        self.count = 0;
    }

    fn set_to_formation(&mut self) {
        self.state = EnemyState::Formation;
        self.speed = 0;
        self.angle = 0;
        self.vangle = 0;
    }

    fn update_attack<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        match self.enemy_type {
            EnemyType::Bee => { self.update_attack_bee(accessor, event_queue); }
            EnemyType::Butterfly => { self.update_attack_butterfly(accessor, event_queue); }
            EnemyType::Owl => { self.update_attack_owl(accessor, event_queue); }
            EnemyType::CapturedFighter => { self.update_attack_bee(accessor, event_queue); }
        }
    }

    fn update_attack_bee<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        match self.attack_step {
            0 => {
                self.speed = 1 * ONE;
                self.angle = 0;
                if self.formation_index.0 < 5 {
                    self.vangle = -4 * ONE;
                } else {
                    self.vangle = 4 * ONE;
                }
                self.attack_step += 1;
                self.count = 0;

                event_queue.push(EventType::EneShot(self.pos, 2 * ONE));
            }
            1 => {
                if (self.vangle < 0 && self.angle <= -160 * ONE) ||
                    (self.vangle > 0 && self.angle >= 160 * ONE)
                {
                    self.vangle = 0;
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            2 => {
                self.count += 1;
                if self.count >= 10 {
                    if self.formation_index.0 < 5 {
                        self.vangle = 1 * ONE / 4;
                    } else {
                        self.vangle = -1 * ONE / 4;
                    }
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            3 => {
                if (self.vangle > 0 && self.angle >= -ANGLE / 2 * ONE) ||
                    (self.vangle < 0 && self.angle <= ANGLE / 2 * ONE)
                {
                    self.vangle = 0;
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            4 => {
                if self.pos.y >= (HEIGHT + 16) * ONE {
                    // TODO: Warp to the top of the screen.
                    self.release_troops(accessor);
                    self.set_to_formation();
                }
            }
            _ => {}
        }
    }

    fn update_attack_butterfly<T: Accessor>(&mut self, accessor: &mut T,
                                            event_queue: &mut EventQueue) {
        self.update_attack_bee(accessor, event_queue);
    }

    fn update_attack_owl<T: Accessor>(&mut self, accessor: &mut T, event_queue: &mut EventQueue) {
        if self.attack_type == 0 {
            self.update_attack_bee(accessor, event_queue);
            return;
        }

        const DLIMIT: i32 = 4 * ONE;
        match self.attack_step {
            0 => {
                self.speed = 3 * ONE / 2;
                self.angle = 0;
                if self.formation_index.0 < 5 {
                    self.vangle = -DLIMIT;
                } else {
                    self.vangle = DLIMIT;
                }

                let player_pos = accessor.get_raw_player_pos();
                self.target_pos = Vec2I::new(player_pos.x, (HEIGHT - 16 - 8 - 88) * ONE);

                self.attack_step += 1;
                self.count = 0;
            }
            1 => {
                let dpos = &self.target_pos - &self.pos;
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

                    self.tractor_beam =
                        Some(TractorBeam::new(&(&self.pos + &Vec2I::new(0, 8 * ONE))));

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
                    } else if tractor_beam.can_capture(accessor.get_raw_player_pos()) {
                        event_queue.push(
                            EventType::CapturePlayer(&self.pos + &Vec2I::new(0, 16 * ONE)));
                        tractor_beam.start_capture();
                        self.capture_state = CaptureState::BeamTracting;
                        self.attack_step = 100;
                        self.count = 0;
                    }
                }
            }
            3 => {
                if self.pos.y >= (HEIGHT + 16) * ONE {
                    // TODO: Warp to the top of the screen.
                    self.set_to_formation();
                }
            }
            // Capture sequence
            100 => {
                if accessor.is_player_captured() {
                    self.tractor_beam.as_mut().unwrap().close_capture();
                    self.capture_state = CaptureState::BeamClosing;
                    self.attack_step += 1;
                    self.count = 0;
                }
            }
            101 => {
                if let Some(tractor_beam) = &self.tractor_beam {
                    if tractor_beam.closed() {
                        let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
                        event_queue.push(EventType::SpawnCapturedFighter(&self.pos + &Vec2I::new(0, 16 * ONE), fi));

                        self.add_troop(fi);

                        // TODO: Turn and back to the formation.
                        self.tractor_beam = None;
                        self.capture_state = CaptureState::Capturing;
                        event_queue.push(EventType::CapturePlayerCompleted);

                        self.speed = 3 * ONE / 2;
                        self.attack_step += 1;
                        self.count = 0;
                    }
                }
            }
            102 => {
                if self.pos.y >= (HEIGHT + 16) * ONE {
                    self.release_troops(accessor);
                    event_queue.push(EventType::CaptureSequenceEnded);
                    // TODO: Warp to the top of the screen.
                    self.set_to_formation();
                }
            }
            _ => {}
        }
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> Option<CollBox> {
        Some(CollBox {
            top_left: &self.pos() - &Vec2I::new(8, 8),
            size: Vec2I::new(12, 12),
        })
    }
}

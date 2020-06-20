use crate::app::consts::*;
use crate::app::enemy::tractor_beam::TractorBeam;
use crate::app::enemy::traj::Traj;
use crate::app::enemy::{Accessor, FormationIndex};
use crate::app::game::{EventQueue, EventType};
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{
    calc_velocity, clamp, diff_angle, quantize_angle, round_up, square,
    ANGLE, ONE, ONE_BIT};

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

#[derive(Debug)]
pub struct DamageResult {
    pub destroyed: bool,
    pub killed: bool,
    pub point: u32,
}

const MAX_TROOPS: usize = 3;

#[derive(Debug)]
pub struct Enemy {
    state: EnemyState,
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
    troops: [Option<FormationIndex>; MAX_TROOPS],
    ghost: bool,
    dead: bool,
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
            troops: Default::default(),
            ghost: false,
            dead: false,
        }
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn raw_pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn get_state(&self) -> EnemyState {
        self.state
    }

    pub fn capture_state(&self) -> CaptureState {
        self.capture_state
    }

    pub fn captured_fighter_index(&self) -> Option<FormationIndex> {
        if self.capture_state == CaptureState::Capturing {
            let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
            if self.troops.iter().flat_map(|x| x)
                .find(|index| **index == fi).is_some()
            {
                return Some(fi);
            }
        }
        None
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }

    pub fn update(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
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
                if !self.update_move_to_formation(accessor) {
                    self.release_troops(accessor);
                    self.set_to_formation();
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

        if self.ghost && !self.dead && !self.live_troops(accessor) {
            self.dead = true;
        }
    }

    fn update_troops(&mut self, add: &Vec2I, angle: i32, accessor: &mut dyn Accessor) {
        for troop_opt in self.troops.iter_mut() {
            if let Some(formation_index) = troop_opt {
                if let Some(troop) = accessor.get_enemy_at_mut(formation_index) {
                    troop.update_troop(add, angle);
                } else {
                    //*troop_opt = None;
                }
            }
        }
    }

    fn update_troop(&mut self, add: &Vec2I, angle: i32) -> bool {
        self.pos += *add;
        self.angle = angle;
        true
    }

    fn release_troops(&mut self, accessor: &mut dyn Accessor) {
        for troop_opt in self.troops.iter_mut() {
            if let Some(index) = troop_opt {
                if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                    enemy.set_to_formation();
                }
                *troop_opt = None;
            }
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        if self.ghost {
            return Ok(());
        }

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

    pub fn set_damage(&mut self, power: u32, accessor: &dyn Accessor,
                      event_queue: &mut EventQueue) -> DamageResult
    {
        if self.life > power {
            self.life -= power;
            DamageResult {destroyed: false, killed: false, point: 0}
        } else {
            self.life = 0;
            if self.live_troops(accessor) {
                self.ghost = true;
            }
            if self.enemy_type == EnemyType::CapturedFighter {
                event_queue.push(EventType::CapturedFighterDestroyed);
            }
            DamageResult {destroyed: true, killed: !self.ghost, point: self.calc_point()}
        }
    }

    fn live_troops(&self, accessor: &dyn Accessor) -> bool {
        self.troops.iter().flat_map(|x| x)
            .filter_map(|index| accessor.get_enemy_at(index))
            .any(|enemy| enemy.enemy_type != EnemyType::CapturedFighter)
    }

    fn calc_point(&self) -> u32 {
        match self.enemy_type {
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
                    let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
                    let count = self.troops.iter().flat_map(|x| x)
                        .filter(|index| **index != fi)
                        .count();
                    match count {
                        0 => 400,
                        1 => 800,
                        2 | _ => 1600,
                    }
                }
            }
            EnemyType::CapturedFighter => { 1000 }
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

    fn update_move_to_formation(&mut self, accessor: &dyn Accessor) -> bool {
        let target = accessor.get_formation_pos(&self.formation_index);
        let diff = &target - &self.pos;
        let sq_distance = square(diff.x >> (ONE_BIT / 2)) + square(diff.y >> (ONE_BIT / 2));
        if sq_distance > square(self.speed >> (ONE_BIT / 2)) {
            const DLIMIT: i32 = 5 * ONE;
            let target_angle_rad = (diff.x as f64).atan2(-diff.y as f64);
            let target_angle = ((target_angle_rad * (((ANGLE * ONE) as f64) / (2.0 * std::f64::consts::PI)) + 0.5).floor() as i32) & (ANGLE * ONE - 1);
            let d = diff_angle(target_angle, self.angle);
            self.angle += clamp(d, -DLIMIT, DLIMIT);
            self.vangle = 0;
            true
        } else {
            self.pos = target;
            self.speed = 0;
            self.angle = 0;
            false
        }
    }

    pub fn set_attack(&mut self, capture_attack: bool, accessor: &mut dyn Accessor) {
        self.state = EnemyState::Attack;
        self.attack_type = if capture_attack { 1 } else { 0 };
        self.attack_step = 0;
        self.count = 0;

        for slot in self.troops.iter_mut() {
            *slot = None;
        }
        if !capture_attack && self.enemy_type == EnemyType::Owl {
            self.choose_troops(accessor);
        }
    }

    fn choose_troops(&mut self, accessor: &mut dyn Accessor) {
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
        self.troops.iter().flat_map(|x| x)
            .for_each(|index| {
                if let Some(enemy) = accessor.get_enemy_at_mut(index) {
                    enemy.set_to_troop();
                }
            });
    }

    fn add_troop(&mut self, formation_index: FormationIndex) -> bool {
        if let Some(slot) = self.troops.iter_mut().find(|x| x.is_none()) {
            *slot = Some(formation_index);
            true
        } else {
            false
        }
    }

    pub fn set_to_troop(&mut self) {
        self.state = EnemyState::Troop;
        self.count = 0;
    }

    fn set_to_formation(&mut self) {
        self.state = EnemyState::Formation;
        self.speed = 0;
        self.angle = 0;
        self.vangle = 0;

        if self.ghost {
            self.dead = true;
        }
    }

    fn update_attack(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
        match self.enemy_type {
            EnemyType::Bee => { self.update_attack_bee(accessor, event_queue); }
            EnemyType::Butterfly => { self.update_attack_butterfly(accessor, event_queue); }
            EnemyType::Owl => { self.update_attack_owl(accessor, event_queue); }
            EnemyType::CapturedFighter => { self.update_attack_bee(accessor, event_queue); }
        }
    }

    fn update_attack_bee(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
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
                }
            }
            4 => {
                if self.pos.y >= (HEIGHT + 8) * ONE {
                    let target_pos = accessor.get_formation_pos(&self.formation_index);
                    let offset = Vec2I::new(target_pos.x - self.pos.x, (-32 - (HEIGHT + 8)) * ONE);
                    self.warp(offset);
                    self.state = EnemyState::MoveToFormation;
                }
            }
            _ => {}
        }
    }

    fn update_attack_butterfly(&mut self, accessor: &mut dyn Accessor,
                               event_queue: &mut EventQueue) {
        self.update_attack_bee(accessor, event_queue);
    }

    fn update_attack_owl(&mut self, accessor: &mut dyn Accessor, event_queue: &mut EventQueue) {
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
                if self.pos.y >= (HEIGHT + 8) * ONE {
                    let target_pos = accessor.get_formation_pos(&self.formation_index);
                    let offset = Vec2I::new(target_pos.x - self.pos.x, (-32 - (HEIGHT + 8)) * ONE);
                    self.warp(offset);
                    self.state = EnemyState::MoveToFormation;
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

                        self.tractor_beam = None;
                        self.capture_state = CaptureState::Capturing;
                        event_queue.push(EventType::CapturePlayerCompleted);

                        self.speed = 3 * ONE / 2;
                        self.attack_step += 1;
                    }
                }
            }
            102 => {
                if !self.update_move_to_formation(accessor) {
                    self.speed = 0;
                    self.attack_step += 1;
                }
            }
            103 => {
                let fi = FormationIndex(self.formation_index.0, self.formation_index.1 - 1);
                let mut done = false;
                if let Some(captured_fighter) = accessor.get_enemy_at_mut(&fi) {
                    let mut y = captured_fighter.pos.y;
                    y -= 1 * ONE;
                    let topy = self.pos.y - 16 * ONE;
                    if y <= topy {
                        y = topy;
                        done = true;
                    }
                    captured_fighter.pos.y = y;
                }
                if done {
                    event_queue.push(EventType::CaptureSequenceEnded);
                    self.release_troops(accessor);
                    self.set_to_formation();
                }
            }
            _ => {}
        }
    }

    fn warp(&mut self, offset: Vec2I) {
        self.pos += offset;
        // No need to modify troops, because offset is calculated from previous position.
    }
}

impl Collidable for Enemy {
    fn get_collbox(&self) -> Option<CollBox> {
        if !self.ghost {
            Some(CollBox {
                top_left: &self.pos() - &Vec2I::new(8, 8),
                size: Vec2I::new(12, 12),
            })
        } else {
            None
        }
    }
}

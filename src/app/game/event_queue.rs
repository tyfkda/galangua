use crate::app::effect::EarnedPointType;
use crate::framework::types::Vec2I;

pub struct EventQueue {
    queue: Vec<EventType>,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            queue: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn get(&self, index: usize) -> &EventType {
        &self.queue[index]
    }

    pub fn spawn_myshot(&mut self, pos: Vec2I, dual: bool) {
        self.queue.push(EventType::MyShot(pos, dual));
    }

    pub fn spawn_ene_shot(&mut self, pos: Vec2I, speed: i32) {
        self.queue.push(EventType::EneShot(pos, speed));
    }

    pub fn add_score(&mut self, add: u32) {
        self.queue.push(EventType::AddScore(add));
    }

    pub fn dead_player(&mut self) {
        self.queue.push(EventType::DeadPlayer);
    }

    pub fn spawn_earn_point(&mut self, point_type: EarnedPointType, pos: Vec2I) {
        self.queue.push(EventType::EarnPoint(point_type, pos));
    }

    pub fn spawn_small_bomb(&mut self, pos: Vec2I) {
        self.queue.push(EventType::SmallBomb(pos));
    }
}

pub enum EventType {
    MyShot(Vec2I, bool),
    AddScore(u32),
    EneShot(Vec2I, i32),
    EarnPoint(EarnedPointType, Vec2I),
    SmallBomb(Vec2I),
    DeadPlayer,
}

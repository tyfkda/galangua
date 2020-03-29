use std::ops::Index;

use crate::app::effect::EarnedPointType;
use crate::framework::types::Vec2I;

pub struct EventQueue {
    queue: Vec<EventType>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn push(&mut self, event: EventType) {
        self.queue.push(event);
    }
}

impl Index<usize> for EventQueue {
    type Output = EventType;
    fn index(&self, i: usize) -> &Self::Output {
        &self.queue[i]
    }
}

pub enum EventType {
    MyShot(Vec2I, bool),
    AddScore(u32),
    EneShot(Vec2I, i32),
    EarnPoint(EarnedPointType, Vec2I),
    SmallBomb(Vec2I),
    DeadPlayer,
    CapturePlayer(Vec2I),
    CapturePlayerCompleted,
    CaptureSequenceEnded,
    RecapturePlayer(Vec2I),
    MovePlayerHomePos,
    RecaptureEnded,
}

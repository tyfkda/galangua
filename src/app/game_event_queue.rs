use super::effect::EarnedPointType;
use super::super::util::types::Vec2I;

pub struct GameEventQueue {
    queue: Vec<GameEvent>,
}

impl GameEventQueue {
    pub fn new() -> GameEventQueue {
        GameEventQueue {
            queue: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn get(&self, index: usize) -> &GameEvent {
        &self.queue[index]
    }

    pub fn spawn_myshot(&mut self, pos: Vec2I, dual: bool) {
        self.queue.push(GameEvent::MyShot(pos, dual));
    }

    pub fn add_score(&mut self, add: u32) {
        self.queue.push(GameEvent::AddScore(add));
    }

    pub fn dead_player(&mut self) {
        self.queue.push(GameEvent::DeadPlayer);
    }

    pub fn spawn_earn_point(&mut self, point_type: EarnedPointType, pos: Vec2I) {
        self.queue.push(GameEvent::EarnPoint(point_type, pos));
    }

    pub fn spawn_small_bomb(&mut self, pos: Vec2I) {
        self.queue.push(GameEvent::SmallBomb(pos));
    }
}

pub enum GameEvent {
    MyShot(Vec2I, bool),
    AddScore(u32),
    EarnPoint(EarnedPointType, Vec2I),
    SmallBomb(Vec2I),
    DeadPlayer,
}

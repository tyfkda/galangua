use super::super::util::types::Vec2I;

pub struct GameEventQueue {
    pub queue: Vec<GameEvent>,
}

impl GameEventQueue {
    pub fn new() -> GameEventQueue {
        GameEventQueue {
            queue: Vec::new(),
        }
    }

    pub fn spawn_myshot(&mut self, pos: Vec2I) {
        self.queue.push(GameEvent::MyShot(pos));
    }

    pub fn add_score(&mut self, add: u32) {
        self.queue.push(GameEvent::AddScore(add));
    }
}

pub enum GameEvent {
    MyShot(Vec2I),
    AddScore(u32),
}

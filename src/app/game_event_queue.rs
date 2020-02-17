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

    pub fn spawn_myshot(&mut self, pos: Vec2I) {
        self.queue.push(GameEvent::MyShot(pos));
    }

    pub fn add_score(&mut self, add: u32) {
        self.queue.push(GameEvent::AddScore(add));
    }

    pub fn dead_player(&mut self) {
        self.queue.push(GameEvent::DeadPlayer);
    }
}

pub enum GameEvent {
    MyShot(Vec2I),
    AddScore(u32),
    DeadPlayer,
}

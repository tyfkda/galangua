pub struct GameEventQueue {
    pub queue: Vec<GameEvent>,
}

impl GameEventQueue {
    pub fn new() -> GameEventQueue {
        GameEventQueue {
            queue: Vec::new(),
        }
    }

    pub fn spawn_myshot(&mut self, x: i32, y: i32) {
        self.queue.push(GameEvent::MyShot(x, y));
    }

    pub fn add_score(&mut self, add: u32) {
        self.queue.push(GameEvent::AddScore(add));
    }
}

pub enum GameEvent {
    MyShot(i32, i32),
    AddScore(u32),
}

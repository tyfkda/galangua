use std::ops::Index;

pub(super) struct EventQueue {
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

#[derive(Clone)]
pub(super) enum EventType {
    AddScore(u32),
    DeadPlayer,
    StarEvent(StarEventType),
    PlaySe(u32, &'static str),
}

#[derive(Copy, Clone)]
pub(super) enum StarEventType {
    Capturing(bool),
    Stop(bool),
}

#[derive(Clone)]
pub struct ScoreHolder {
    pub score: u32,
    pub high_score: u32,
}

impl ScoreHolder {
    pub fn new(high_score: u32) -> Self {
        Self {
            score: 0,
            high_score,
        }
    }

    pub fn reset_score(&mut self) {
        self.score = 0;
    }

    pub fn add_score(&mut self, add: u32) {
        self.score = self.score.saturating_add(add);
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScoreHolder {
    pub score: u32,
    pub high_score: u32,
}

impl ScoreHolder {
    pub fn reset_score(&mut self) {
        self.score = 0;
    }

    pub fn add_score(&mut self, add: u32) {
        self.score += add;
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }
}

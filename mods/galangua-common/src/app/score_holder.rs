use crate::framework::RendererTrait;

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

    pub fn draw(&self, renderer: &mut impl RendererTrait, show_1up: bool) {
        renderer.set_texture_color_mod("font", 255, 0, 0);
        if show_1up {
            renderer.draw_str("font", 2 * 8, 0 * 8, "1UP");
        }
        renderer.draw_str("font", 9 * 8, 0 * 8, "HIGH SCORE");
        renderer.set_texture_color_mod("font", 255, 255, 255);

        const MAX_DISP_SCORE: u32 = 9999999;
        let score = std::cmp::min(self.score, MAX_DISP_SCORE);
        renderer.draw_str("font", 0 * 8, 1 * 8, &format!("{:6}0", score / 10));
        let high_score = std::cmp::min(self.high_score, MAX_DISP_SCORE);
        renderer.draw_str("font", 10 * 8, 1 * 8, &format!("{:6}0", high_score / 10));
    }
}

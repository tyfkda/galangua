use crate::app::consts::*;
use crate::app::game::EventQueue;
use crate::app::util::{CollBox, Collidable};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{clamp, round_up, ANGLE, ONE};
use crate::util::pad::{Pad, PAD_A, PAD_L, PAD_R};

#[derive(PartialEq)]
enum State {
    Normal,
    Dual,
    Dead,
    Capturing,
    Captured,
    CaptureCompleted,
}

pub struct Player {
    pos: Vec2I,
    state: State,
    angle: i32,
    capture_pos: Vec2I,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2I::new(WIDTH / 2, HEIGHT - 16 - 8) * ONE,
            state: State::Normal,
            angle: 0,
            capture_pos: Vec2I::new(0, 0),
        }
    }

    pub fn restart(&mut self) {
        self.state = State::Normal;
        self.pos = Vec2I::new(WIDTH / 2, HEIGHT - 16 - 8) * ONE;
    }

    pub fn update(&mut self, pad: &Pad, event_queue: &mut EventQueue) {
        match self.state {
            State::Normal | State::Dual => {
                self.update_normal(pad, event_queue);
            }
            State::Dead => {
            }
            State::Capturing => {
                self.update_capture();
            }
            State::Captured | State::CaptureCompleted => {}
        }
    }

    pub fn update_normal(&mut self, pad: &Pad, event_queue: &mut EventQueue) {
        if pad.is_pressed(PAD_L) {
            self.pos.x -= 2 * ONE;
            if self.pos.x < 8 * ONE {
                self.pos.x = 8 * ONE;
            }
        }
        if pad.is_pressed(PAD_R) {
            self.pos.x += 2 * ONE;

            let right = if self.dual() { (WIDTH - 8 - 16) * ONE } else { (WIDTH - 8) * ONE };
            if self.pos.x > right {
                self.pos.x = right;
            }
        }
        if pad.is_trigger(PAD_A) {
            event_queue.spawn_myshot(Vec2I::new(self.pos.x, self.pos.y - 8 * ONE), self.dual());
        }
    }

    pub fn update_capture(&mut self) {
        const D: i32 = 1 * ONE;
        let d = self.capture_pos - self.pos;
        self.pos.x += clamp(d.x, -D, D);
        self.pos.y += clamp(d.y, -D, D);
        self.angle += ANGLE * ONE / 32;

        if d.x == 0 && d.y == 0 {
            self.state = State::Captured;
            self.angle = 0;
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        match self.state {
            State::Normal | State::Dual => {
                let pos = self.pos();
                renderer.draw_sprite("fighter", pos + Vec2I::new(-8, -8))?;
                if self.dual() {
                    renderer.draw_sprite("fighter", pos + Vec2I::new(-8 + 16, -8))?;
                }
            }
            State::Capturing => {
                let pos = self.pos();
                let angle = calc_display_angle(self.angle);
                renderer.draw_sprite_rot("fighter", pos + Vec2I::new(-8, -8), angle, Some(Vec2I::new(7, 10)))?;
            }
            State::Captured => {
                let pos = self.pos();
                renderer.draw_sprite("captured", pos + Vec2I::new(-8, -8))?;
            }
            State::CaptureCompleted | State::Dead => {}
        }

        Ok(())
    }

    fn dual(&self) -> bool {
        self.state == State::Dual
    }

    pub fn active(&self) -> bool {
        self.state == State::Normal || self.state == State::Dual
    }

    pub fn pos(&self) -> Vec2I {
        round_up(&self.pos)
    }

    pub fn get_raw_pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn dual_pos(&self) -> Option<Vec2I> {
        if self.dual() {
            Some(self.pos() + Vec2I::new(16, 0))
        } else {
            None
        }
    }

    pub fn dual_collbox(&self) -> Option<CollBox> {
        if self.dual() {
            Some(CollBox {
                top_left: self.pos() + Vec2I::new(8, -8),
                size: Vec2I::new(16, 16),
            })
        } else {
            None
        }
    }

    pub fn crash(&mut self, pos: &Vec2I) -> bool {
        if self.dual() {
            if pos.x < self.pos().x + 16 / 2 {
                // Abandan left ship.
                self.pos.x += 16 * ONE;
            }
            self.state = State::Normal;
            false
        } else {
            self.state = State::Dead;
            true
        }
    }

    pub fn start_capture(&mut self, capture_pos: Vec2I) {
        self.state = State::Capturing;
        self.capture_pos = capture_pos;
        self.angle = 0;
    }

    pub fn complete_capture(&mut self) {
        self.state = State::CaptureCompleted;
    }

    pub fn set_dual(&mut self) {
        if self.state == State::Normal {
            self.state = State::Dual;
        }
    }
}

impl Collidable for Player {
    fn get_collbox(&self) -> CollBox {
        CollBox {
            top_left: self.pos() - Vec2I::new(8, 8),
            size: Vec2I::new(16, 16),
        }
    }
}

fn calc_display_angle(angle: i32) -> f64 {
    // Quantize.
    let div = 16;
    let angle = (angle + ANGLE * ONE / div / 2) & (ANGLE * ONE - (ANGLE * ONE / div));
    let angle: f64 = (angle as f64) * (360.0 / ((ANGLE * ONE) as f64));

    angle
}

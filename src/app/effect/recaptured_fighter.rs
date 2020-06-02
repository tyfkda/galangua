use crate::app::consts::*;
use crate::app::game::{EventQueue, EventType};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{clamp, round_up, ANGLE, ONE};

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Rotate,
    SlideHorz,
    SlideDown,
    Done,
}

pub struct RecapturedFighter {
    pos: Vec2I,
    state: State,
    angle: i32,
}

impl RecapturedFighter {
    pub fn new(pos: Vec2I) -> Self {
        Self {
            pos: pos,
            state: State::Rotate,
            angle: 0,
        }
    }

    pub fn update(&mut self, player_living: bool, event_queue: &mut EventQueue) {
        match self.state {
            State::Rotate => {
                self.angle += ANGLE * ONE / 32;
                if self.angle >= ANGLE * ONE * 4 {
                    self.state = State::SlideHorz;
                    event_queue.push(EventType::MovePlayerHomePos);
                }
            }
            State::SlideHorz => {
                let x = if player_living { (WIDTH / 2 + 8) * ONE } else { WIDTH / 2 * ONE };
                let speed = 1 * ONE;
                self.pos.x += clamp(x - self.pos.x, -speed, speed);
                if self.pos.x == x {
                    self.state = State::SlideDown;
                }
            }
            State::SlideDown => {
                let y = (HEIGHT - 16 - 8) * ONE;
                let speed = 1 * ONE;
                self.pos.y += clamp(y - self.pos.y, -speed, speed);
                if self.pos.y == y {
                    self.state = State::Done
                }
            }
            State::Done => {}
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        let pos = round_up(&self.pos);
        match self.state {
            State::Rotate => {
                let angle = ((self.angle / (ANGLE * ONE / 16)) * 360 / (ANGLE / 16)) as f64;
                renderer.draw_sprite_rot("fighter", &pos + &Vec2I::new(-8, -8), angle, Some(Vec2I::new(7, 10)))?;
            }
            _ => {
                renderer.draw_sprite("fighter", &pos + &Vec2I::new(-8, -8))?;
            }
        }

        Ok(())
    }

    pub fn done(&self) -> bool {
        self.state == State::Done
    }
}

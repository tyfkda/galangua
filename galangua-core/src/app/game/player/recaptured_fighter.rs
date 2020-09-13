use crate::app::consts::*;
use crate::app::game::{EventQueue, EventType};
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{clamp, quantize_angle, round_up, ANGLE, ONE};

use super::Accessor;

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Rotate,
    SlideHorz,
    SlideDown,
    Done,
}

pub(super) struct RecapturedFighter {
    pos: Vec2I,
    state: State,
    angle: i32,
}

impl RecapturedFighter {
    pub(super) fn new(pos: &Vec2I) -> Self {
        Self {
            pos: *pos,
            state: State::Rotate,
            angle: 0,
        }
    }

    pub(super) fn update<A: Accessor>(
        &mut self, player_living: bool, accessor: &A, event_queue: &mut EventQueue,
    ) {
        match self.state {
            State::Rotate => {
                self.angle += ANGLE * ONE / ANGLE_DIV;
                if self.angle >= ANGLE * ONE * 4 && accessor.is_no_attacker() {
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

    pub(super) fn draw<R>(&self, renderer: &mut R)
    where
        R: RendererTrait,
    {
        let pos = round_up(&self.pos);
        match self.state {
            State::Rotate => {
                let angle = quantize_angle(self.angle, ANGLE_DIV);
                //renderer.draw_sprite_rot("rustacean", &(&pos + &Vec2I::new(-8, -8)), angle, Some(Vec2I::new(7, 10)));
                renderer.draw_sprite_rot("rustacean", &(&pos + &Vec2I::new(-8, -8)), angle, None);
            }
            _ => {
                renderer.draw_sprite("rustacean", &(&pos + &Vec2I::new(-8, -8)));
            }
        }
    }

    pub(super) fn done(&self) -> bool {
        self.state == State::Done
    }
}

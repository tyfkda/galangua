use crate::app::consts::*;
use crate::app::game::manager::EventType;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{clamp, quantize_angle, round_vec, ANGLE, ONE};

use super::Accessor;

const SPRITE_NAME: &str = "rustacean";

#[derive(Clone, Copy, PartialEq)]
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
    pub(super) fn new(pos: &Vec2I, angle: i32) -> Self {
        Self {
            pos: *pos,
            state: State::Rotate,
            angle: angle & (ANGLE * ONE - 1),
        }
    }

    pub(super) fn update<A: Accessor>(&mut self, player_living: bool, accessor: &mut A) {
        const DANGLE: i32 = ANGLE * ONE / ANGLE_DIV;
        const SPEED: i32 = 2 * ONE;
        match self.state {
            State::Rotate => {
                self.angle += DANGLE;
                if self.angle >= ANGLE * ONE * 4 && accessor.is_no_attacker() {
                    self.state = State::SlideHorz;
                    self.angle = 0;
                    accessor.push_event(EventType::MovePlayerHomePos);
                }
            }
            State::SlideHorz => {
                let x = CENTER_X + if player_living { 8 * ONE } else { 0 };
                self.pos.x += clamp(x - self.pos.x, -SPEED, SPEED);
                if self.pos.x == x {
                    self.state = State::SlideDown;
                }
            }
            State::SlideDown => {
                self.pos.y += clamp(PLAYER_Y - self.pos.y, -SPEED, SPEED);
                if self.pos.y == PLAYER_Y {
                    self.state = State::Done;
                }
            }
            State::Done => {}
        }
    }

    pub(super) fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        let pos = round_vec(&self.pos);
        match self.state {
            State::Rotate => {
                let angle = quantize_angle(self.angle, ANGLE_DIV);
                renderer.draw_sprite_rot(SPRITE_NAME, &(&pos + &Vec2I::new(-8, -8)), angle, None);
            }
            _ => {
                renderer.draw_sprite(SPRITE_NAME, &(&pos + &Vec2I::new(-8, -8)));
            }
        }
    }

    pub(super) fn done(&self) -> bool {
        self.state == State::Done
    }
}

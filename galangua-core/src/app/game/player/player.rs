use crate::app::consts::*;
use crate::app::game::manager::EventType;
use crate::app::util::collision::{CollBox, Collidable};
use crate::framework::types::{Vec2I, ZERO_VEC};
use crate::framework::RendererTrait;
use crate::util::math::{calc_velocity, clamp, quantize_angle, round_vec, ANGLE, ONE};
use crate::util::pad::{Pad, PadBit};

use super::recaptured_fighter::RecapturedFighter;
use super::Accessor;

const SPRITE_NAME: &str = "rustacean";
const SPRITE_NAME_CAPTURED: &str = "rustacean_captured";

#[derive(PartialEq)]
enum State {
    Normal,
    Dead,
    Capturing,
    Captured,
    EscapeCapturing,
    MoveHomePos,
}

pub struct Player {
    pos: Vec2I,
    state: State,
    dual: bool,
    angle: i32,
    capture_pos: Vec2I,
    recaptured_fighter: Option<RecapturedFighter>,
    shot_enable: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2I::new(CENTER_X, PLAYER_Y),
            state: State::Normal,
            dual: false,
            angle: 0,
            capture_pos: ZERO_VEC,
            recaptured_fighter: None,
            shot_enable: true,
        }
    }

    pub fn restart(&mut self) {
        self.state = State::Normal;
        self.pos = Vec2I::new(CENTER_X, PLAYER_Y);
    }

    pub fn set_shot_enable(&mut self, value: bool) {
        self.shot_enable = value;
    }

    pub fn update<A: Accessor>(&mut self, pad: &Pad, accessor: &mut A) {
        match self.state {
            State::Normal => self.update_normal(pad, accessor),
            State::Capturing => self.update_capture(pad, accessor),
            State::EscapeCapturing => {
                const D: i32 = 1 * ONE;
                self.pos.y += D;
                if self.pos.y >= PLAYER_Y {
                    self.pos.y = PLAYER_Y;
                    self.state = State::Normal;
                    accessor.push_event(EventType::EscapeEnded);
                }
            }
            State::MoveHomePos => {
                let x = CENTER_X - 8 * ONE;
                let speed = 2 * ONE;
                self.pos.x += clamp(x - self.pos.x, -speed, speed);
                if self.pos.x == x {
                    if self.recaptured_fighter.as_ref().unwrap().done() {
                        self.dual = true;
                        self.state = State::Normal;
                        self.recaptured_fighter = None;
                        accessor.push_event(EventType::RecaptureEnded(true));
                    }
                }
            }
            State::Dead | State::Captured => {}
        }

        if let Some(recaptured_fighter) = &mut self.recaptured_fighter {
            recaptured_fighter.update(self.state != State::Dead, accessor);
            if self.state == State::Dead && recaptured_fighter.done() {
                self.pos.x = CENTER_X;
                self.state = State::Normal;
                self.recaptured_fighter = None;
                accessor.push_event(EventType::RecaptureEnded(false));
            }
        }
    }

    pub fn update_normal<A: Accessor>(&mut self, pad: &Pad, accessor: &mut A) {
        if pad.is_pressed(PadBit::L) {
            self.pos.x -= PLAYER_SPEED;
            let left = 8 * ONE;
            if self.pos.x < left {
                self.pos.x = left;
            }
        }
        if pad.is_pressed(PadBit::R) {
            self.pos.x += PLAYER_SPEED;
            let right = if self.dual { (WIDTH - 8 - 16) * ONE } else { (WIDTH - 8) * ONE };
            if self.pos.x > right {
                self.pos.x = right;
            }
        }
        self.fire_bullet(pad, accessor);
    }

    pub fn update_capture<A: Accessor>(&mut self, pad: &Pad, accessor: &mut A) {
        const D: i32 = 1 * ONE;
        let d = &self.capture_pos - &self.pos;
        self.pos.x += clamp(d.x, -D, D);
        self.pos.y += clamp(d.y, -D, D);
        self.angle += ANGLE * ONE / ANGLE_DIV;

        self.fire_bullet(pad, accessor);

        if d.x == 0 && d.y == 0 {
            self.state = State::Captured;
            self.angle = 0;
        }
    }

    fn fire_bullet<A: Accessor>(&mut self, pad: &Pad, accessor: &mut A) {
        if self.shot_enable && pad.is_trigger(PadBit::A) {
            let pos = &self.pos + &calc_velocity(self.angle, 4 * ONE);
            accessor.push_event(EventType::MyShot(pos, self.dual, self.angle));
        }
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        match self.state {
            State::Normal | State::EscapeCapturing | State::MoveHomePos => {
                let pos = round_vec(&self.pos);
                renderer.draw_sprite(SPRITE_NAME, &(&pos + &Vec2I::new(-8, -8)));
                if self.dual {
                    renderer.draw_sprite(SPRITE_NAME, &(&pos + &Vec2I::new(-8 + 16, -8)));
                }
            }
            State::Capturing => {
                let pos = round_vec(&self.pos);
                let angle = quantize_angle(self.angle, ANGLE_DIV);
                renderer.draw_sprite_rot(SPRITE_NAME, &(&pos + &Vec2I::new(-8, -8)), angle, None);
            }
            State::Captured => {
                let pos = round_vec(&self.pos);
                renderer.draw_sprite(SPRITE_NAME_CAPTURED, &(&pos + &Vec2I::new(-8, -8)));
            }
            State::Dead => {}
        }

        if let Some(recaptured_fighter) = &self.recaptured_fighter {
            recaptured_fighter.draw(renderer);
        }
    }

    pub fn pos(&self) -> &Vec2I {
        &self.pos
    }

    pub fn dual_pos(&self) -> Option<Vec2I> {
        if self.dual {
            Some(&self.pos + &Vec2I::new(16 * ONE, 0))
        } else {
            None
        }
    }

    pub fn dual_collbox(&self) -> Option<CollBox> {
        if self.dual && self.state == State::Normal {
            Some(CollBox {
                top_left: &round_vec(&self.pos) + &Vec2I::new(-4 + 16, -4),
                size: Vec2I::new(8, 8),
            })
        } else {
            None
        }
    }

    pub fn crash(&mut self, dual: bool) -> bool {
        if self.dual {
            if !dual {
                // Abandan left ship.
                self.pos.x += 16 * ONE;
            }
            self.dual = false;
            false
        } else {
            assert!(!dual);
            self.state = State::Dead;
            true
        }
    }

    pub fn start_capture(&mut self, capture_pos: &Vec2I) {
        self.state = State::Capturing;
        self.capture_pos = *capture_pos;
        self.angle = 0;
    }

    pub fn is_captured(&self) -> bool {
        self.state == State::Captured
    }

    pub fn complete_capture(&mut self) {
        self.state = State::Dead;
    }

    pub fn escape_capturing(&mut self) {
        self.state = State::EscapeCapturing;
        self.angle = 0;
    }

    pub fn start_recapture_effect(&mut self, pos: &Vec2I, angle: i32) {
        self.recaptured_fighter = Some(RecapturedFighter::new(pos, angle));
    }

    pub fn start_move_home_pos(&mut self) {
        if self.state != State::Dead {
            self.state = State::MoveHomePos;
        }
    }
}

impl Collidable for Player {
    fn get_collbox(&self) -> Option<CollBox> {
        if self.state == State::Normal {
            Some(CollBox {
                top_left: &round_vec(&self.pos) - &Vec2I::new(4, 4),
                size: Vec2I::new(8, 8),
            })
        } else {
            None
        }
    }
}

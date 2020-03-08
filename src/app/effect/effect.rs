use sdl2::rect::Rect;
use std::cmp::min;

use super::super::super::framework::Renderer;
use super::super::super::util::types::Vec2I;

pub enum Effect {
    EarnedPoint(EarnedPoint),
    SmallBomb(SmallBomb),
}

impl Effect {
    pub fn update(&mut self) -> bool {
        match self {
            Effect::EarnedPoint(x) => x.update(),
            Effect::SmallBomb(x) => x.update(),
        }
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) -> Result<(), String> {
        match self {
            Effect::EarnedPoint(x) => x.draw(renderer),
            Effect::SmallBomb(x) => x.draw(renderer),
        }
    }
}

//

#[derive(Clone, Copy)]
pub enum EarnedPointType {
    Point1600,
    Point800,
    Point400,
    Point150,
}

pub struct EarnedPoint {
    point_type: EarnedPointType,
    pos: Vec2I,
    frame_count: u32,
}

impl EarnedPoint {
    pub fn new(point_type: EarnedPointType, pos: Vec2I) -> EarnedPoint {
        EarnedPoint {
            point_type,
            pos,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;

        self.frame_count < 30
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) -> Result<(), String> {
        let rect: Rect;
        match self.point_type {
            EarnedPointType::Point1600 => { rect = Rect::new(32, 0, 16, 8); },
            EarnedPointType::Point800  => { rect = Rect::new(48, 0, 16, 8); },
            EarnedPointType::Point400  => { rect = Rect::new(32, 8, 16, 8); },
            EarnedPointType::Point150  => { rect = Rect::new(48, 8, 16, 8); },
        }

        renderer.draw_texture("chr",
                              Some(rect),
                              Some(Rect::new((self.pos.x - 8) * 2, (self.pos.y - 4) * 2, 16 * 2, 8 * 2)))?;

        Ok(())
    }
}

//

pub struct SmallBomb {
    pos: Vec2I,
    frame_count: u32,
}

impl SmallBomb {
    pub fn new(pos: Vec2I) -> SmallBomb {
        SmallBomb {
            pos,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;

        self.frame_count < 15
    }

    pub fn draw(&self, renderer: &mut dyn Renderer) -> Result<(), String> {
        let pat = min(self.frame_count / 4, 2) as i32;

        renderer.draw_texture("chr",
                              Some(Rect::new(pat * 16, 64, 16, 16)),
                              Some(Rect::new((self.pos.x - 8) * 2, (self.pos.y - 8) * 2, 16 * 2, 16 * 2)))?;

        Ok(())
    }
}

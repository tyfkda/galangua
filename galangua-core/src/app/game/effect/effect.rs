use std::cmp::min;

use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::round_up;

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

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        match self {
            Effect::EarnedPoint(x) => x.draw(renderer),
            Effect::SmallBomb(x) => x.draw(renderer),
        }
    }
}

//

#[derive(Clone, Copy, Debug)]
pub enum EarnedPointType {
    Point1600,
    Point1000,
    Point800,
    Point400,
}

pub struct EarnedPoint {
    point_type: EarnedPointType,
    pos: Vec2I,
    frame_count: u32,
}

impl EarnedPoint {
    pub fn new(point_type: EarnedPointType, pos: &Vec2I) -> Self {
        Self {
            point_type,
            pos: round_up(&pos),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;

        self.frame_count < 30
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let sprite: &str = match self.point_type {
            EarnedPointType::Point1600 => "pts1600",
            EarnedPointType::Point1000 => "pts1000",
            EarnedPointType::Point800  => "pts800",
            EarnedPointType::Point400  => "pts400",
        };
        renderer.draw_sprite(sprite, &(&self.pos + &Vec2I::new(-8, -4)))?;
        Ok(())
    }
}

//

pub struct SmallBomb {
    pos: Vec2I,
    frame_count: u32,
}

impl SmallBomb {
    pub fn new(pos: &Vec2I) -> Self {
        Self {
            pos: round_up(&pos),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;

        self.frame_count < 15
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let pat = min(self.frame_count / 4, 2) as usize;
        let table = ["small_bomb1", "small_bomb2", "small_bomb3"];
        renderer.draw_sprite(table[pat], &(&self.pos + &Vec2I::new(-8, -8)))?;
        Ok(())
    }
}

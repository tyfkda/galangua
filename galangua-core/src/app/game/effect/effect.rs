use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::round_up;

pub enum Effect {
    EarnedPoint(EarnedPoint),
    EnemyExplosion(EnemyExplosion),
    PlayerExplosion(PlayerExplosion),
}

impl Effect {
    pub fn update(&mut self) -> bool {
        match self {
            Effect::EarnedPoint(x) => x.update(),
            Effect::EnemyExplosion(x) => x.update(),
            Effect::PlayerExplosion(x) => x.update(),
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        match self {
            Effect::EarnedPoint(x) => x.draw(renderer),
            Effect::EnemyExplosion(x) => x.draw(renderer),
            Effect::PlayerExplosion(x) => x.draw(renderer),
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

const ENEMY_EXPLOSION_SPRITE_TABLE: [&str; 5] = ["ene_exp1", "ene_exp2", "ene_exp3", "ene_exp4", "ene_exp5"];
const ENEMY_EXPLOSION_FRAME: u32 = 4;

pub struct EnemyExplosion {
    pos: Vec2I,
    frame_count: u32,
}

impl EnemyExplosion {
    pub fn new(pos: &Vec2I) -> Self {
        Self {
            pos: round_up(&pos),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;

        self.frame_count < ENEMY_EXPLOSION_SPRITE_TABLE.len() as u32 * ENEMY_EXPLOSION_FRAME
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let pat = (self.frame_count / ENEMY_EXPLOSION_FRAME) as usize;
        renderer.draw_sprite(ENEMY_EXPLOSION_SPRITE_TABLE[pat], &(&self.pos + &Vec2I::new(-16, -16)))?;
        Ok(())
    }
}

//

const PLAYER_EXPLOSION_SPRITE_TABLE: [&str; 4] = ["pl_exp1", "pl_exp2", "pl_exp3", "pl_exp4"];
const PLAYER_EXPLOSION_FRAME: u32 = 8;

pub struct PlayerExplosion {
    pos: Vec2I,
    frame_count: u32,
}

impl PlayerExplosion {
    pub fn new(pos: &Vec2I) -> Self {
        Self {
            pos: round_up(&pos),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frame_count += 1;

        self.frame_count < PLAYER_EXPLOSION_SPRITE_TABLE.len() as u32 * PLAYER_EXPLOSION_FRAME
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let pat = (self.frame_count / PLAYER_EXPLOSION_FRAME) as usize;
        renderer.draw_sprite(PLAYER_EXPLOSION_SPRITE_TABLE[pat], &(&self.pos + &Vec2I::new(-16, -16)))?;
        Ok(())
    }
}

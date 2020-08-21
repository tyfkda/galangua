use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::round_up;

pub enum Effect {
    SequentialSpriteAnime(SequentialSpriteAnime),
}

impl Effect {
    pub fn create_earned_point(point_type: EarnedPointType, pos: &Vec2I) -> Self {
        Effect::SequentialSpriteAnime(
            SequentialSpriteAnime::new(
                &round_up(&pos) + &Vec2I::new(-8, -4),
                &EARNED_POINT_SPRITE_TABLE[point_type as usize],
                EARNED_POINT_FRAME))
    }

    pub fn create_enemy_explosion(pos: &Vec2I) -> Self {
        Effect::SequentialSpriteAnime(
            SequentialSpriteAnime::new(
                &round_up(&pos) + &Vec2I::new(-16, -16),
                &ENEMY_EXPLOSION_SPRITE_TABLE,
                ENEMY_EXPLOSION_FRAME))
    }

    pub fn create_player_explosion(pos: &Vec2I) -> Self {
        Effect::SequentialSpriteAnime(
            SequentialSpriteAnime::new(
                &round_up(&pos) + &Vec2I::new(-16, -16),
                &PLAYER_EXPLOSION_SPRITE_TABLE,
                PLAYER_EXPLOSION_FRAME))
    }

    pub fn update(&mut self) -> bool {
        match self {
            Effect::SequentialSpriteAnime(x) => x.update(),
        }
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        match self {
            Effect::SequentialSpriteAnime(x) => x.draw(renderer),
        }
    }
}

//

pub struct SequentialSpriteAnime {
    pos: Vec2I,
    sprites: &'static [&'static str],
    frame_wait: u32,
    frame: u32,
    count: u32,
}

impl SequentialSpriteAnime {
    pub fn new(pos: Vec2I, sprites: &'static [&str], frame_wait: u32) -> Self {
        Self {
            pos,
            sprites,
            frame_wait,
            frame: 0,
            count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.count += 1;
        if self.count >= self.frame_wait {
            self.count = 0;
            self.frame += 1;
        }

        self.frame < self.sprites.len() as u32
    }

    pub fn draw<R>(&self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        let sprite = self.sprites[self.frame as usize];
        renderer.draw_sprite(sprite, &self.pos)
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

const EARNED_POINT_FRAME: u32 = 64;
const EARNED_POINT_SPRITE_TABLE: [[&str; 1]; 4] = [
    ["pts1600"],
    ["pts1000"],
    ["pts800"],
    ["pts400"],
];

const ENEMY_EXPLOSION_SPRITE_TABLE: [&str; 5] = ["ene_exp1", "ene_exp2", "ene_exp3", "ene_exp4", "ene_exp5"];
const ENEMY_EXPLOSION_FRAME: u32 = 4;

const PLAYER_EXPLOSION_SPRITE_TABLE: [&str; 4] = ["pl_exp1", "pl_exp2", "pl_exp3", "pl_exp4"];
const PLAYER_EXPLOSION_FRAME: u32 = 8;

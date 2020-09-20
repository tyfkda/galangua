use crate::app::consts::*;
use crate::app::game::enemy::EnemyType;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;
use crate::util::math::{quantize_angle, round_vec};

pub enum Effect {
    SequentialSpriteAnime(SequentialSpriteAnime),
    RotSprite(RotSprite),
}

const FLASH_ENEMY_SPRITE_NAMES: [&str; 4] = [
    "gopher_flash",
    "dman_flash",
    "cpp_flash",
    "rustacean_flash",
];

const FLASH_ENEMY_FRAME: u32 = 2;

impl Effect {
    pub fn create_earned_point(point_type: EarnedPointType, pos: &Vec2I) -> Self {
        Effect::SequentialSpriteAnime(
            SequentialSpriteAnime::new(
                &round_vec(&pos) + &Vec2I::new(-8, -4),
                &EARNED_POINT_SPRITE_TABLE[point_type as usize],
                EARNED_POINT_FRAME, 0))
    }

    pub fn create_flash_enemy(pos: &Vec2I, angle: i32, enemy_type: EnemyType) -> Self {
        let sprite_name = FLASH_ENEMY_SPRITE_NAMES[enemy_type as usize];
        Effect::RotSprite(
            RotSprite::new(
                &round_vec(&pos) + &Vec2I::new(-8, -8),
                quantize_angle(angle, ANGLE_DIV),
                sprite_name,
                FLASH_ENEMY_FRAME))
    }

    pub fn create_enemy_explosion(pos: &Vec2I) -> Self {
        Effect::SequentialSpriteAnime(
            SequentialSpriteAnime::new(
                &round_vec(&pos) + &Vec2I::new(-16, -16),
                &ENEMY_EXPLOSION_SPRITE_TABLE,
                ENEMY_EXPLOSION_FRAME, 2))
    }

    pub fn create_player_explosion(pos: &Vec2I) -> Self {
        Effect::SequentialSpriteAnime(
            SequentialSpriteAnime::new(
                &round_vec(&pos) + &Vec2I::new(-16, -16),
                &PLAYER_EXPLOSION_SPRITE_TABLE,
                PLAYER_EXPLOSION_FRAME, 0))
    }

    pub fn update(&mut self) -> bool {
        match self {
            Effect::SequentialSpriteAnime(x) => x.update(),
            Effect::RotSprite(x) => x.update(),
        }
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        match self {
            Effect::SequentialSpriteAnime(x) => x.draw(renderer),
            Effect::RotSprite(x) => x.draw(renderer),
        }
    }
}

//

pub struct SequentialSpriteAnime {
    pos: Vec2I,
    sprites: &'static [&'static str],
    delay: i32,
    frame_wait: u32,
    frame: u32,
    count: u32,
}

impl SequentialSpriteAnime {
    pub fn new(pos: Vec2I, sprites: &'static [&str], frame_wait: u32, delay: i32) -> Self {
        Self {
            pos,
            sprites,
            frame_wait,
            delay,
            frame: 0,
            count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        if self.delay >= 0 {
            self.delay -= 1;
            return true;
        }

        self.count += 1;
        if self.count >= self.frame_wait {
            self.count = 0;
            self.frame += 1;
        }

        self.frame < self.sprites.len() as u32
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        if self.delay >= 0 {
            return;
        }

        let sprite = self.sprites[self.frame as usize];
        renderer.draw_sprite(sprite, &self.pos);
    }
}

//

pub struct RotSprite {
    pos: Vec2I,
    angle: u8,
    sprite_name: &'static str,
    duration: u32,
    count: u32,
}

impl RotSprite {
    pub fn new(pos: Vec2I, angle: u8, sprite_name: &'static str, duration: u32) -> Self {
        Self {
            pos,
            angle,
            sprite_name,
            duration,
            count: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.count += 1;
        self.count <= self.duration
    }

    pub fn draw<R: RendererTrait>(&self, renderer: &mut R) {
        renderer.draw_sprite_rot(self.sprite_name, &self.pos, self.angle, None);
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

pub fn to_earned_point_type(point: u32) -> Option<EarnedPointType> {
    match point {
        1600 => Some(EarnedPointType::Point1600),
        1000 => Some(EarnedPointType::Point1000),
        800 => Some(EarnedPointType::Point800),
        400 => Some(EarnedPointType::Point400),
        _ => None,
    }
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

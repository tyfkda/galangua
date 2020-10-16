use legion::systems::CommandBuffer;

use galangua_common::app::game::effect_table::*;
use galangua_common::app::game::EarnedPointType;
use galangua_common::framework::types::Vec2I;

use crate::app::components::*;

pub fn new_seqanime(sprites: &'static [&'static str], frame_wait: u32) -> SequentialSpriteAnime {
    SequentialSpriteAnime(sprites, frame_wait, 0)
}

pub fn update_seqanime(anime: &mut SequentialSpriteAnime) -> Option<&'static str> {
    let sprites = &mut anime.0;
    let frame_wait = anime.1;
    let count = &mut anime.2;
    if sprites.len() == 0 {
        return None;
    }
    *count += 1;
    if *count >= frame_wait {
        *count = 0;
        *sprites = &sprites[1..];
        if sprites.len() == 0 {
            return None;
        }
    }
    Some(sprites[0])
}

pub fn create_enemy_explosion_effect(
    pos: &Vec2I, commands: &mut CommandBuffer,
) {
    let anime_table = &ENEMY_EXPLOSION_SPRITE_TABLE;
    let sprite_name = anime_table[0];
    let offset = Vec2I::new(-16, -16);
    commands.push((
        Posture(pos.clone(), 0),
        new_seqanime(anime_table, ENEMY_EXPLOSION_FRAME),
        SpriteDrawable { sprite_name, offset },
    ));
}

pub fn create_player_explosion_effect(
    pos: &Vec2I, commands: &mut CommandBuffer,
) {
    let anime_table = &PLAYER_EXPLOSION_SPRITE_TABLE;
    let sprite_name = anime_table[0];
    let offset = Vec2I::new(-16, -16);
    commands.push((
        Posture(pos.clone(), 0),
        new_seqanime(anime_table, PLAYER_EXPLOSION_FRAME),
        SpriteDrawable { sprite_name, offset },
    ));
}

pub fn create_earned_piont_effect(
    point_type: EarnedPointType,
    pos: &Vec2I,
    commands: &mut CommandBuffer,
) {
    let anime_table = &EARNED_POINT_SPRITE_TABLE[point_type as usize];
    let sprite_name = anime_table[0];
    let offset = Vec2I::new(-8, -4);
    commands.push((
        Posture(pos.clone(), 0),
        new_seqanime(anime_table, EARNED_POINT_FRAME),
        SpriteDrawable { sprite_name, offset },
    ));
}

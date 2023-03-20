use legion::systems::CommandBuffer;
use legion::*;

use galangua_common::app::game::effect_table::*;
use galangua_common::app::game::{EarnedPointType, EnemyType};
use galangua_common::framework::types::Vec2I;

use crate::app::components::*;

pub fn new_seqanime(sprites: &'static [&'static str], offset: Vec2I, frame_wait: u32, delay: u32) -> SequentialSpriteAnime {
    SequentialSpriteAnime {sprites, frame_wait, delay: delay + 1, offset, count: 0}
}

pub fn update_seqanime(anime: &mut SequentialSpriteAnime, drawable: Option<&mut SpriteDrawable>, entity: Entity, commands: &mut CommandBuffer) {
    if anime.delay > 0 {
        anime.delay -= 1;
        if anime.delay > 0 {
            return;
        }

        // Add SpriteDrawable component.
        if drawable.is_none() {
            commands.add_component(entity, SpriteDrawable { sprite_name: anime.sprites[0], offset: anime.offset });
        }
    }

    anime.count += 1;
    if anime.count >= anime.frame_wait {
        anime.count = 0;
        anime.sprites = &anime.sprites[1..];
        if anime.sprites.is_empty() {
            commands.remove(entity);
            return;
        }
        drawable.unwrap().sprite_name = anime.sprites[0];
    }
}

const FLASH_ENEMY_ANIME_TABLE: [&[&str]; 4] = [
    &FLASH_BEE_ANIME_TABLE,
    &FLASH_BUTTERFLY_ANIME_TABLE,
    &FLASH_OWL_ANIME_TABLE,
    &FLASH_CAPTURED_FIGHTER_ANIME_TABLE,
];
const FLASH_BEE_ANIME_TABLE: [&str; 1] = ["gopher_flash"];
const FLASH_BUTTERFLY_ANIME_TABLE: [&str; 1] = ["dman_flash"];
const FLASH_OWL_ANIME_TABLE: [&str; 1] = ["cpp_flash"];
const FLASH_CAPTURED_FIGHTER_ANIME_TABLE: [&str; 1] = ["rustacean_flash"];

pub fn create_flash_enemy_effect(
    pos: &Vec2I, angle: i32, enemy_type: EnemyType, commands: &mut CommandBuffer,
) {
    let anime_table = FLASH_ENEMY_ANIME_TABLE[enemy_type as usize];
    let sprite_name = anime_table[0];
    let offset = Vec2I::new(-8, -8);
    commands.push((
        Posture(*pos, angle),
        new_seqanime(anime_table, offset, FLASH_ENEMY_FRAME, 0),
        SpriteDrawable { sprite_name, offset },
    ));
}

pub fn create_enemy_explosion_effect(
    pos: &Vec2I, delay: u32, commands: &mut CommandBuffer,
) {
    assert!(delay > 0);
    let anime_table = &ENEMY_EXPLOSION_SPRITE_TABLE;
    let offset = Vec2I::new(-16, -16);
    commands.push((
        Posture(*pos, 0),
        new_seqanime(anime_table, offset, ENEMY_EXPLOSION_FRAME, delay),
    ));
}

pub fn create_player_explosion_effect(
    pos: &Vec2I, commands: &mut CommandBuffer,
) {
    let anime_table = &PLAYER_EXPLOSION_SPRITE_TABLE;
    let sprite_name = anime_table[0];
    let offset = Vec2I::new(-16, -16);
    commands.push((
        Posture(*pos, 0),
        new_seqanime(anime_table, offset, PLAYER_EXPLOSION_FRAME, 0),
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
        Posture(*pos, 0),
        new_seqanime(anime_table, offset, EARNED_POINT_FRAME, 0),
        SpriteDrawable { sprite_name, offset },
    ));
}

use crate::app::game::EarnedPointType;

pub const FLASH_ENEMY_SPRITE_NAMES: [&str; 4] = [
    "gopher_flash",
    "dman_flash",
    "cpp_flash",
    "rustacean_flash",
];

//

pub fn to_earned_point_type(point: u32) -> Option<EarnedPointType> {
    match point {
        1600 => Some(EarnedPointType::Point1600),
        1000 => Some(EarnedPointType::Point1000),
        800 => Some(EarnedPointType::Point800),
        400 => Some(EarnedPointType::Point400),
        _ => None,
    }
}

pub const EARNED_POINT_FRAME: u32 = 64;
pub const EARNED_POINT_SPRITE_TABLE: [[&str; 1]; 4] = [
    ["pts1600"],
    ["pts1000"],
    ["pts800"],
    ["pts400"],
];

pub const ENEMY_EXPLOSION_SPRITE_TABLE: [&str; 5] = ["ene_exp1", "ene_exp2", "ene_exp3", "ene_exp4", "ene_exp5"];
pub const ENEMY_EXPLOSION_FRAME: u32 = 4;

pub const PLAYER_EXPLOSION_SPRITE_TABLE: [&str; 4] = ["pl_exp1", "pl_exp2", "pl_exp3", "pl_exp4"];
pub const PLAYER_EXPLOSION_FRAME: u32 = 8;

use crate::util::math::ONE;

pub const WIDTH: i32 = 224;
pub const HEIGHT: i32 = 288;
pub const ANGLE_DIV: i32 = 24;

pub const KEY_HIGH_SCORE: &str = "highScore";
pub const DEFAULT_HIGH_SCORE: u32 = 1000;

pub const DEFAULT_LEFT_SHIP: u32 = 3;

pub const CENTER_X: i32 = WIDTH * ONE / 2;
pub const PLAYER_Y: i32 = (HEIGHT - 16 - 8) * ONE;
pub const PLAYER_SPEED: i32 = 3 * ONE / 2;
pub const MYSHOT_SPEED: i32 = 6 * ONE;

pub const ENE_SHOT_SPEED1: i32 = 25 * ONE / 10;
pub const ENE_SHOT_SPEED2: i32 = 40 * ONE / 10;

pub const EXTEND_FIRST_SCORE: u32 = 20_000;
pub const EXTEND_AFTER_SCORE: u32 = 50_000;

pub const RUSH_THRESHOLD: u32 = 5;

pub const BASE_VOLUME: f32 = 1.0 / 4.0;

pub const CHANNEL_COUNT: u32 = 3;
pub const CH_SHOT: u32 = 0;
pub const CH_BOMB: u32 = 0;
pub const CH_ATTACK: u32 = 1;
pub const CH_JINGLE: u32 = 2;

pub const SE_COUNT_STAGE: &str = "assets/audio/se_get_1";
pub const SE_MYSHOT: &str = "assets/audio/se_pyuun";
pub const SE_DAMAGE: &str = "assets/audio/se_pow_1";
pub const SE_BOMB_ZAKO: &str = "assets/audio/se_zugyan";
pub const SE_BOMB_PLAYER: &str = "assets/audio/se_zugyan";
pub const SE_BOMB_CAPTURED: &str = "assets/audio/se_gyuin";
pub const SE_ATTACK_START: &str = "assets/audio/attack_start";
pub const SE_TRACTOR_BEAM1: &str = "assets/audio/se_pipipi_2";
pub const SE_TRACTOR_BEAM2: &str = "assets/audio/se_pipipi_1";
pub const SE_EXTEND_SHIP: &str = "assets/audio/jingle_1up";
pub const SE_RECAPTURE: &str = "assets/audio/jingle_item01";

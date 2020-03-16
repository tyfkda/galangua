use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::collections::HashMap;

use super::sprite_sheet::SpriteSheet;
use super::texture_manager::TextureManager;

pub trait RendererTrait {
    fn load_textures(&mut self, base_path: &str, filenames: &Vec<&str>) -> Result<(), String>;
    fn set_sprite_sheet(&mut self, sprite_sheet: HashMap<String, SpriteSheet>);
    fn get_mut_texture_manager(&mut self) -> &mut TextureManager;
    fn clear(&mut self);
    fn present(&mut self);
    fn set_texture_color_mod(&mut self, tex_name: &str, r: u8, g: u8, b: u8);
    fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str) -> Result<(), String>;
    fn draw_sprite(&mut self, sprite_name: &str, pos: Point) -> Result<(), String>;
    fn draw_sprite_rot(&mut self, sprite_name: &str, pos: Point, angle: f64, center: Option<Point>) -> Result<(), String>;
    fn set_draw_color(&mut self, color: Color);
    fn fill_rect(&mut self, dst: Option<Rect>) -> Result<(), String>;
}

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};

use super::texture_manager::TextureManager;

pub trait Renderer {
    fn load_textures(&mut self, base_path: &str, filenames: &Vec<&str>) -> Result<(), String>;
    fn get_mut_texture_manager(&mut self) -> &mut TextureManager;
    fn clear(&mut self);
    fn present(&mut self);
    fn set_texture_color_mod(&mut self, tex_name: &str, r: u8, g: u8, b: u8);
    fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str) -> Result<(), String>;
    fn draw_texture(&mut self, tex_name: &str, src: Option<Rect>, dst: Option<Rect>) -> Result<(), String>;
    fn draw_texture_ex(&mut self, tex_name: &str, src: Option<Rect>, dst: Option<Rect>,
                       angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool) -> Result<(), String>;
    fn set_draw_color(&mut self, color: Color);
    fn fill_rect(&mut self, dst: Option<Rect>) -> Result<(), String>;
}

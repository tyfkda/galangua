use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;

use super::texture_manager::TextureManager;

pub struct Renderer {
    texture_manager: TextureManager,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            texture_manager: TextureManager::new(),
        }
    }

    pub fn get_mut_texture_manager(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
    }

    pub fn draw_texture(&self, canvas: &mut WindowCanvas, tex_name: &str, src: Option<Rect>, dst: Option<Rect>) -> Result<(), String> {
        if let Some(texture) = self.texture_manager.get(tex_name) {
            canvas.copy(&texture,
                        src, dst)?;
            return Ok(());
        }
        Err(format!("No texture: {}", tex_name))
    }

    pub fn draw_texture_ex(&self, canvas: &mut WindowCanvas, tex_name: &str, src: Option<Rect>, dst: Option<Rect>,
                           angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool) -> Result<(), String>
    {
        if let Some(texture) = self.texture_manager.get(tex_name) {
            canvas.copy_ex(&texture,
                           src, dst, angle, center, flip_horizontal, flip_vertical)?;
            return Ok(());
        }
        Err(format!("No texture: {}", tex_name))
    }

    pub fn set_draw_color(&self, canvas: &mut WindowCanvas, color: Color) {
        canvas.set_draw_color(color);
    }

    pub fn fill_rect(&self, canvas: &mut WindowCanvas, dst: Option<Rect>) -> Result<(), String> {
        canvas.fill_rect(dst)?;
        Ok(())
    }
}

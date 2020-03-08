use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;

use super::texture_manager::TextureManager;

pub struct Renderer {
    canvas: WindowCanvas,
    texture_manager: TextureManager,
}

impl Renderer {
    pub fn new(canvas: WindowCanvas) -> Renderer {
        Renderer {
            canvas,
            texture_manager: TextureManager::new(),
        }
    }

    pub fn load_textures(&mut self, base_path: &str, filenames: &Vec<&str>) -> Result<(), String> {
        self.texture_manager.load(&mut self.canvas, base_path, filenames)
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn set_texture_color_mod(&mut self, tex_name: &str, r: u8, g: u8, b: u8) {
        if let Some(texture) = self.texture_manager.get_mut(tex_name) {
            texture.set_color_mod(r, g, b);
        }
    }

    pub fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str) -> Result<(), String> {
        if let Some(texture) = self.texture_manager.get_mut(tex_name) {
            let w = 16;
            let h = 16;
            let mut x = x;

            for c in text.chars() {
                let u: i32 = ((c as i32) - (' ' as i32)) % 16 * 8;
                let v: i32 = ((c as i32) - (' ' as i32)) / 16 * 8;
                self.canvas.copy(&texture,
                                 Some(Rect::new(u, v, 8, 8)),
                                 Some(Rect::new(x, y, w, h)))?;
                x += w as i32;
            }
            Ok(())
        } else {
            Err(format!("No texture: {}", tex_name))
        }
    }

    pub fn draw_texture(&mut self, tex_name: &str, src: Option<Rect>, dst: Option<Rect>) -> Result<(), String> {
        if let Some(texture) = self.texture_manager.get(tex_name) {
            self.canvas.copy(&texture,
                             src, dst)?;
            return Ok(());
        }
        return Err(format!("No texture: {}", tex_name));
    }

    pub fn draw_texture_ex(&mut self, tex_name: &str, src: Option<Rect>, dst: Option<Rect>,
                           angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool) -> Result<(), String>
    {
        if let Some(texture) = self.texture_manager.get(tex_name) {
            self.canvas.copy_ex(&texture,
                                src, dst, angle, center, flip_horizontal, flip_vertical)?;
            return Ok(());
        }
        return Err(format!("No texture: {}", tex_name));
    }

    pub fn set_draw_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }

    pub fn fill_rect(&mut self, dst: Option<Rect>) -> Result<(), String> {
        self.canvas.fill_rect(dst)?;
        Ok(())
    }
}

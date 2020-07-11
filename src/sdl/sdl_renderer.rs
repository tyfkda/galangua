use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use std::collections::HashMap;

use crate::framework::sprite_sheet::SpriteSheet;
use crate::framework::types::Vec2I;
use crate::framework::RendererTrait;

use super::sdl_texture_manager::SdlTextureManager;

pub struct SdlRenderer {
    canvas: WindowCanvas,
    texture_manager: SdlTextureManager,
    sprite_sheet: HashMap<String, SpriteSheet>,
    scale: i32,
}

impl SdlRenderer {
    pub fn new(canvas: WindowCanvas, scale: u32) -> Self {
        Self {
            canvas,
            texture_manager: SdlTextureManager::new(),
            sprite_sheet: HashMap::new(),
            scale: scale as i32,
        }
    }
}

impl RendererTrait for SdlRenderer {
    fn load_textures(&mut self, base_path: &str, filenames: &[&str]) -> Result<(), String> {
        self.texture_manager.load(&mut self.canvas, base_path, filenames)
    }

    fn set_sprite_sheet(&mut self, sprite_sheet: HashMap<String, SpriteSheet>) {
        self.sprite_sheet = sprite_sheet;
    }

    fn clear(&mut self) {
        self.canvas.clear();
    }

    fn present(&mut self) {
        self.canvas.present();
    }

    fn set_texture_color_mod(&mut self, tex_name: &str, r: u8, g: u8, b: u8) {
        if let Some(texture) = self.texture_manager.get_mut(tex_name) {
            texture.set_color_mod(r, g, b);
        }
    }

    fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str) -> Result<(), String> {
        if let Some(texture) = self.texture_manager.get_mut(tex_name) {
            let w = (8 * self.scale) as u32;
            let h = (8 * self.scale) as u32;
            let mut x = x * self.scale;
            let y = y * self.scale;

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

    fn draw_sprite(&mut self, sprite_name: &str, pos: &Vec2I) -> Result<(), String> {
        let sheet = self.sprite_sheet.get(sprite_name)
            .expect(&format!("No sprite: {}", sprite_name));
        let mut pos = *pos;
        if let Some(trimmed) = &sheet.trimmed {
            pos.x += trimmed.sprite_source_size.x;
            pos.y += trimmed.sprite_source_size.y;
        }

        let texture = self.texture_manager.get(&sheet.texture)
            .expect(&format!("No texture: {}", sheet.texture));
        self.canvas.copy(&texture,
                         Some(Rect::new(sheet.frame.x, sheet.frame.y,
                                        sheet.frame.w, sheet.frame.h)),
                         Some(Rect::new(pos.x * self.scale, pos.y * self.scale,
                                        sheet.frame.w * self.scale as u32,
                                        sheet.frame.h * self.scale as u32)))?;
        Ok(())
    }

    fn draw_sprite_rot(&mut self, sprite_name: &str, pos: &Vec2I, angle: u8,
                       center: Option<&Vec2I>) -> Result<(), String> {
        let sheet = self.sprite_sheet.get(sprite_name)
            .expect(&format!("No sprite: {}", sprite_name));
        let mut pos = *pos;
        if let Some(trimmed) = &sheet.trimmed {
            pos.x += trimmed.sprite_source_size.x;
            pos.y += trimmed.sprite_source_size.y;
        }

        let texture = self.texture_manager.get(&sheet.texture)
            .expect(&format!("No texture: {}", sheet.texture));
        let center = center.map(|v| Point::new(v.x * self.scale, v.y * self.scale));
        self.canvas.copy_ex(&texture,
                            Some(Rect::new(sheet.frame.x, sheet.frame.y,
                                           sheet.frame.w, sheet.frame.h)),
                            Some(Rect::new(pos.x * self.scale, pos.y * self.scale,
                                           sheet.frame.w * self.scale as u32,
                                           sheet.frame.h * self.scale as u32)),
                            (angle as f64) * (360.0 / 256.0), center, false, false)?;
        Ok(())
    }

    fn set_draw_color(&mut self, r: u8, g: u8, b: u8) {
        self.canvas.set_draw_color(Color::RGB(r, g, b));
    }

    fn fill_rect(&mut self, dst: Option<[&Vec2I; 2]>) -> Result<(), String> {
        if let Some(rect) = dst {
            self.canvas.fill_rect(Some(Rect::new(rect[0].x * self.scale, rect[0].y * self.scale,
                                                 (rect[1].x * self.scale) as u32,
                                                 (rect[1].y * self.scale) as u32)))?;
        } else {
            self.canvas.fill_rect(None)?;
        }
        Ok(())
    }
}

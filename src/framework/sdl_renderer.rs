use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use std::collections::HashMap;

use crate::framework::RendererTrait;
use crate::framework::sprite_sheet::SpriteSheet;
use crate::framework::texture_manager::TextureManager;

pub struct SdlRenderer {
    canvas: WindowCanvas,
    texture_manager: TextureManager,
    sprite_sheet: HashMap<String, SpriteSheet>,
}

impl SdlRenderer {
    pub fn new(canvas: WindowCanvas) -> SdlRenderer {
        SdlRenderer {
            canvas,
            texture_manager: TextureManager::new(),
            sprite_sheet: HashMap::new(),
        }
    }
}

impl RendererTrait for SdlRenderer {
    fn load_textures(&mut self, base_path: &str, filenames: &Vec<&str>) -> Result<(), String> {
        self.texture_manager.load(&mut self.canvas, base_path, filenames)
    }

    fn set_sprite_sheet(&mut self, sprite_sheet: HashMap<String, SpriteSheet>) {
        self.sprite_sheet = sprite_sheet;
    }

    fn get_mut_texture_manager(&mut self) -> &mut TextureManager {
        &mut self.texture_manager
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

    fn draw_sprite(&mut self, sprite_name: &str, mut pos: Point) -> Result<(), String> {
        let sheet = self.sprite_sheet.get(sprite_name).expect(&format!("No sprite: {}", sprite_name));
        if let Some(trimmed) = &sheet.trimmed {
            pos.x += trimmed.sprite_source_size.x;
            pos.y += trimmed.sprite_source_size.y;
        }

        let texture = self.texture_manager.get(&sheet.texture).expect(&format!("No texture: {}", sheet.texture));
        self.canvas.copy(&texture,
                         Some(Rect::new(sheet.frame.x, sheet.frame.y, sheet.frame.w, sheet.frame.h)),
                         Some(Rect::new(pos.x * 2, pos.y * 2, sheet.frame.w * 2, sheet.frame.h * 2)))?;
        Ok(())
    }

    fn draw_sprite_rot(&mut self, sprite_name: &str, mut pos: Point, angle: f64, center: Option<Point>) -> Result<(), String> {
        let sheet = self.sprite_sheet.get(sprite_name).expect(&format!("No sprite: {}", sprite_name));
        if let Some(trimmed) = &sheet.trimmed {
            pos.x += trimmed.sprite_source_size.x;
            pos.y += trimmed.sprite_source_size.y;
        }

        let texture = self.texture_manager.get(&sheet.texture).expect(&format!("No texture: {}", sheet.texture));
        self.canvas.copy_ex(&texture,
                            Some(Rect::new(sheet.frame.x, sheet.frame.y, sheet.frame.w, sheet.frame.h)),
                            Some(Rect::new(pos.x * 2, pos.y * 2, sheet.frame.w * 2, sheet.frame.h * 2)),
                            angle, center, false, false)?;
        Ok(())
    }

    fn set_draw_color(&mut self, r: u8, g: u8, b: u8) {
        self.canvas.set_draw_color(Color::RGB(r, g, b));
    }

    fn fill_rect(&mut self, dst: Option<Rect>) -> Result<(), String> {
        if let Some(rect) = dst {
            self.canvas.fill_rect(Some(Rect::new(rect.x * 2, rect.y * 2, (rect.w * 2) as u32, (rect.h * 2) as u32)))?;
        } else {
            self.canvas.fill_rect(None)?;
        }
        Ok(())
    }
}

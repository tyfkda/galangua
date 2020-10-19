use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use std::collections::HashMap;

use galangua_core::framework::sprite_sheet::SpriteSheet;
use galangua_core::framework::types::Vec2I;
use galangua_core::framework::RendererTrait;

use super::sdl_texture_manager::SdlTextureManager;

pub struct SdlRenderer {
    canvas: WindowCanvas,
    texture_manager: SdlTextureManager,
    sprite_sheet: SpriteSheet,
    tex_color_map: HashMap<String, (u8, u8, u8)>,
}

impl SdlRenderer {
    pub fn new(mut canvas: WindowCanvas, logical_size: (u32, u32)) -> Self {
        canvas.set_logical_size(logical_size.0, logical_size.1)
            .expect("set_logical_size failed");

        Self {
            canvas,
            texture_manager: SdlTextureManager::new(),
            sprite_sheet: SpriteSheet::default(),
            tex_color_map: HashMap::new(),
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}

impl RendererTrait for SdlRenderer {
    fn load_textures(&mut self, base_path: &str, filenames: &[&str]) {
        self.texture_manager.load(&mut self.canvas, base_path, filenames)
            .expect("load_textures failed");
    }

    fn load_sprite_sheet(&mut self, filename: &str) {
        let text = std::fs::read_to_string(filename)
            .expect("load sprite sheet failed");
        self.sprite_sheet.load_sprite_sheet(&text);
    }

    fn clear(&mut self) {
        self.canvas.clear();
    }

    fn set_texture_color_mod(&mut self, tex_name: &str, r: u8, g: u8, b: u8) {
        let color = (r, g, b);
        if self.tex_color_map.contains_key(tex_name) {
            if self.tex_color_map[tex_name] == color {
                return;
            }
        }
        self.tex_color_map.insert(tex_name.to_string(), color);

        if let Some(texture) = self.texture_manager.get_mut(tex_name) {
            texture.set_color_mod(r, g, b);
        }
    }

    fn set_sprite_texture_color_mod(&mut self, sprite_name: &str, r: u8, g: u8, b: u8) {
        if let Some((_sheet, tex_name)) = self.sprite_sheet.get(sprite_name) {
            let color = (r, g, b);
            if self.tex_color_map.contains_key(tex_name) {
                if self.tex_color_map[tex_name] == color {
                    return;
                }
            }
            self.tex_color_map.insert(tex_name.to_string(), color);

            if let Some(texture) = self.texture_manager.get_mut(tex_name) {
                texture.set_color_mod(r, g, b);
            }
        }
    }

    fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str) {
        let texture = self.texture_manager.get_mut(tex_name)
            .expect("No texture");
        let w = 8;
        let h = 8;
        let mut x = x;

        for c in text.chars() {
            let u: i32 = ((c as i32) - (' ' as i32)) % 16 * 8;
            let v: i32 = ((c as i32) - (' ' as i32)) / 16 * 8;
            self.canvas.copy(&texture,
                             Some(Rect::new(u, v, 8, 8)),
                             Some(Rect::new(x, y, w, h)))
                .expect("copy failed");
            x += w as i32;
        }
    }

    fn draw_sprite(&mut self, sprite_name: &str, pos: &Vec2I) {
        let (sheet, tex_name) = self.sprite_sheet.get(sprite_name)
            .expect("No sprite");
        let pos = sheet.trim_pos(pos);

        let texture = self.texture_manager.get(tex_name)
            .expect("No texture");
        self.canvas.copy(&texture,
                         Some(Rect::new(sheet.frame.x, sheet.frame.y,
                                        sheet.frame.w, sheet.frame.h)),
                         Some(Rect::new(pos.x, pos.y,
                                        sheet.frame.w as u32,
                                        sheet.frame.h as u32)))
            .expect("copy failed");
    }

    fn draw_sprite_rot(&mut self, sprite_name: &str, pos: &Vec2I, angle: u8,
                       center: Option<&Vec2I>) {
        let (sheet, tex_name) = self.sprite_sheet.get(sprite_name)
            .expect("No sprite");
        let pos = sheet.trim_pos(pos);

        let texture = self.texture_manager.get(tex_name)
            .expect("No texture");
        let center = center.map(|v| Point::new(v.x, v.y));
        self.canvas.copy_ex(&texture,
                            Some(Rect::new(sheet.frame.x, sheet.frame.y,
                                           sheet.frame.w, sheet.frame.h)),
                            Some(Rect::new(pos.x, pos.y,
                                           sheet.frame.w as u32,
                                           sheet.frame.h as u32)),
                            (angle as f64) * (360.0 / 256.0), center, false, false)
            .expect("copy_ex failed");
    }

    fn set_draw_color(&mut self, r: u8, g: u8, b: u8) {
        self.canvas.set_draw_color(Color::RGB(r, g, b));
    }

    fn fill_rect(&mut self, dst: Option<[&Vec2I; 2]>) {
        if let Some(rect) = dst {
            self.canvas.fill_rect(Some(Rect::new(rect[0].x, rect[0].y,
                                                 rect[1].x as u32, rect[1].y as u32)))
                .expect("fill_rect failed");
        } else {
            self.canvas.fill_rect(None)
                .expect("fill_rect failed");
        }
    }
}

extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

use super::draw_util::draw_str;
use super::game_manager::GameManager;
use super::super::framework::{App};
use super::super::framework::texture_manager::TextureManager;
use super::super::util::fps_calc::{FpsCalc};
use super::super::util::pad::{Pad};

pub struct GaragaApp {
    pad: Pad,
    fps_calc: FpsCalc,
    texture_manager: TextureManager,
    game_manager: GameManager,
}

impl GaragaApp {
    pub fn new() -> GaragaApp {
        GaragaApp {
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            texture_manager: TextureManager::new(),
            game_manager: GameManager::new(),
        }
    }
}

impl App for GaragaApp {
    fn on_key_down(&mut self, keycode: Keycode) {
        self.pad.on_key_down(keycode);
    }

    fn on_key_up(&mut self, keycode: Keycode) {
        self.pad.on_key_up(keycode);
    }

    fn init(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        self.texture_manager.load(canvas, "assets", &vec!["chr.png", "font.png"])?;

        Ok(())
    }

    fn update(&mut self) {
        self.pad.update();
        self.game_manager.update(&self.pad);
    }

    fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.clear();

        self.game_manager.draw(canvas, &mut self.texture_manager)?;

        if let Some(font_texture) = self.texture_manager.get_mut("font") {
            font_texture.set_color_mod(128, 128, 128);
            draw_str(canvas, &font_texture, 16 * 23, 0, &format!("FPS{:2}", self.fps_calc.fps()))?;
            font_texture.set_color_mod(255, 255, 255);
        }

        canvas.present();

        self.fps_calc.update();

        Ok(())
    }
}

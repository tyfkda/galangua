use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;

use super::consts;
use super::game::GameManager;
use super::util::draw_str;
use super::super::framework::{App, Renderer, SdlAppFramework};
use super::super::util::fps_calc::FpsCalc;
use super::super::util::pad::Pad;

pub struct GaragaApp {
    pad: Pad,
    fps_calc: FpsCalc,
    renderer: Renderer,
    game_manager: GameManager,
}

impl GaragaApp {
    pub fn new() -> GaragaApp {
        GaragaApp {
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            renderer: Renderer::new(),
            game_manager: GameManager::new(),
        }
    }

    pub fn generate_and_run() -> Result<(), String> {
        let app = GaragaApp::new();
        let mut framework = SdlAppFramework::new(
            "Garaga",
            (consts::WIDTH as u32) * 2,
            (consts::HEIGHT as u32) * 2,
            Box::new(app))?;
        framework.run()
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
        self.renderer.get_mut_texture_manager().load(canvas, "assets", &vec!["chr.png", "font.png"])?;

        Ok(())
    }

    fn update(&mut self) {
        self.pad.update();
        self.game_manager.update(&self.pad);
    }

    fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        self.game_manager.draw(canvas, &mut self.renderer)?;

        if let Some(font_texture) = self.renderer.get_mut_texture_manager().get_mut("font") {
            font_texture.set_color_mod(128, 128, 128);
            draw_str(canvas, &font_texture, 16 * 23, 0, &format!("FPS{:2}", self.fps_calc.fps()))?;
            font_texture.set_color_mod(255, 255, 255);
        }

        canvas.present();

        self.fps_calc.update();

        Ok(())
    }
}

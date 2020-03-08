use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use super::consts;
use super::game::GameManager;
use super::super::framework::{App, Renderer, SdlAppFramework};
use super::super::util::fps_calc::FpsCalc;
use super::super::util::pad::Pad;

pub struct GaragaApp {
    pad: Pad,
    fps_calc: FpsCalc,
    game_manager: GameManager,
}

impl GaragaApp {
    pub fn new() -> GaragaApp {
        GaragaApp {
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            game_manager: GameManager::new(),
        }
    }

    pub fn generate_and_run() -> Result<(), String> {
        let app = GaragaApp::new();
        let mut framework = SdlAppFramework::new(
            Box::new(app))?;
        framework.run("Garaga",
            (consts::WIDTH as u32) * 2,
            (consts::HEIGHT as u32) * 2,
)
    }
}

impl App for GaragaApp {
    fn on_key_down(&mut self, keycode: Keycode) {
        self.pad.on_key_down(keycode);
    }

    fn on_key_up(&mut self, keycode: Keycode) {
        self.pad.on_key_up(keycode);
    }

    fn init(&mut self, renderer: &mut Renderer) -> Result<(), String> {
        renderer.load_textures("assets", &vec!["chr.png", "font.png"])?;

        Ok(())
    }

    fn update(&mut self) {
        self.pad.update();
        self.game_manager.update(&self.pad);
    }

    fn draw(&mut self, renderer: &mut Renderer) -> Result<(), String> {
        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.clear();

        self.game_manager.draw(renderer)?;

        renderer.set_texture_color_mod("font", 128, 128, 128);
        renderer.draw_str("font", 16 * 23, 0, &format!("FPS{:2}", self.fps_calc.fps()))?;
        renderer.set_texture_color_mod("font", 255, 255, 255);

        renderer.present();

        self.fps_calc.update();

        Ok(())
    }
}

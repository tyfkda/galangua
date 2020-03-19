use sdl2::keyboard::Keycode;

use crate::app::game::GameManager;
use crate::app::sprite_sheet::load_sprite_sheet;
use crate::framework::{AppTrait, RendererTrait};
use crate::util::fps_calc::FpsCalc;
use crate::util::pad::Pad;

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
}

impl<Renderer: RendererTrait> AppTrait<Renderer> for GaragaApp {
    fn on_key_down(&mut self, keycode: Keycode) {
        self.pad.on_key_down(keycode);
    }

    fn on_key_up(&mut self, keycode: Keycode) {
        self.pad.on_key_up(keycode);
    }

    fn init(&mut self, renderer: &mut Renderer) -> Result<(), String>
        where Renderer: RendererTrait
    {
        renderer.load_textures("assets", &vec!["chr.png", "font.png"])?;

        let sprite_sheet = load_sprite_sheet("assets/chr.json").expect("sprite sheet");
        renderer.set_sprite_sheet(sprite_sheet);

        Ok(())
    }

    fn update(&mut self) {
        self.pad.update();
        self.game_manager.update(&self.pad);
    }

    fn draw(&mut self, renderer: &mut Renderer) -> Result<(), String>
        where Renderer: RendererTrait
    {
        renderer.set_draw_color(0, 0, 0);
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

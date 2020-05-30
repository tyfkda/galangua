use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::effect::StarManager;
use crate::app::game::GameManager;
use crate::app::sprite_sheet::load_sprite_sheet;
use crate::framework::{AppTrait, RendererTrait};
use crate::util::fps_calc::FpsCalc;
use crate::util::pad::{Pad, PAD_A, PAD_START};

#[derive(PartialEq)]
enum AppState {
    Title,
    Game,
}

pub struct GaragaApp {
    state: AppState,
    count: u32,
    pad: Pad,
    fps_calc: FpsCalc,
    game_manager: GameManager,
    star_manager: Rc<RefCell<StarManager>>,
    frame_count: u32,

    paused: bool,
}

impl GaragaApp {
    pub fn new() -> Self {
        let star_manager = Rc::new(RefCell::new(StarManager::new()));
        Self {
            state: AppState::Title,
            count: 0,
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            game_manager: GameManager::new(Rc::clone(&star_manager)),
            star_manager: star_manager,
            frame_count: 0,

            paused: false,
        }
    }
}

impl<R: RendererTrait> AppTrait<R> for GaragaApp {
    fn on_key_down(&mut self, keycode: Keycode) {
        self.pad.on_key_down(keycode);
    }

    fn on_key_up(&mut self, keycode: Keycode) {
        self.pad.on_key_up(keycode);
    }

    fn init(&mut self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        renderer.load_textures("assets", &vec!["chr.png", "font.png"])?;

        let sprite_sheet = load_sprite_sheet("assets/chr.json").expect("sprite sheet");
        renderer.set_sprite_sheet(sprite_sheet);

        Ok(())
    }

    fn update(&mut self) {
        self.pad.update();

        if self.state != AppState::Title {
            if self.pad.is_trigger(PAD_START) {
                self.paused = !self.paused;
            }
            if self.paused {
                return;
            }
        }

        self.star_manager.borrow_mut().update();

        match self.state {
            AppState::Title => {
                self.count = self.count.wrapping_add(1);

                if self.pad.is_trigger(PAD_A) {
                    self.game_manager.restart();
                    self.state = AppState::Game;
                    self.frame_count = 0;
                }
            }
            AppState::Game => {
                self.frame_count += 1;
                self.game_manager.update(&self.pad);
                if self.game_manager.is_finished() {
                    self.state = AppState::Title;
                    self.count = 0;
                }
            }
        }
    }

    fn draw(&mut self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        self.star_manager.borrow().draw(renderer)?;
        match self.state {
            AppState::Title => {
                renderer.set_texture_color_mod("font", 255, 255, 255);
                renderer.draw_str("font", 11 * 16, 8 * 16, "GARAGA")?;

                if self.count & 32 == 0 {
                    renderer.draw_str("font", 2 * 16, 25 * 16, "PRESS SPACE KEY TO START")?;
                }
            }
            AppState::Game => {
                self.game_manager.draw(renderer)?;
            }
        }

        renderer.set_texture_color_mod("font", 255, 0, 0);
        if (self.frame_count & 31) < 16 || self.state != AppState::Game {
            renderer.draw_str("font", 16 * 2, 16 * 0, "1UP")?;
        }
        renderer.draw_str("font", 16 * 9, 16 * 0, "HIGH SCORE")?;
        renderer.set_texture_color_mod("font", 255, 255, 255);
        renderer.draw_str("font", 16 * 0, 16 * 1, &format!("{:6}0", self.game_manager.score() / 10))?;
        renderer.draw_str("font", 16 * 10, 16 * 1, &format!("{:6}0", self.game_manager.high_score() / 10))?;

        renderer.set_texture_color_mod("font", 128, 128, 128);
        renderer.draw_str("font", 16 * 23, 0, &format!("FPS{:2}", self.fps_calc.fps()))?;

        renderer.present();

        self.fps_calc.update();

        Ok(())
    }
}

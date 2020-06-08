use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::effect::StarManager;
use crate::app::game::GameManager;
use crate::app::sprite_sheet::load_sprite_sheet;
use crate::framework::{AppTrait, RendererTrait};
use crate::util::fps_calc::FpsCalc;
use crate::util::pad::{Pad, PadBit};

#[derive(PartialEq)]
enum AppState {
    Title,
    Game,
}

pub struct GalanguaApp {
    state: AppState,
    count: u32,
    pad: Pad,
    fps_calc: FpsCalc,
    game_manager: GameManager,
    star_manager: Rc<RefCell<StarManager>>,
    frame_count: u32,

    paused: bool,
}

impl GalanguaApp {
    pub fn new() -> Self {
        let star_manager = Rc::new(RefCell::new(StarManager::new()));
        Self {
            state: AppState::Title,
            count: 0,
            pad: Pad::new(),
            fps_calc: FpsCalc::new(),
            game_manager: GameManager::new(Rc::clone(&star_manager)),
            star_manager,
            frame_count: 0,

            paused: false,
        }
    }

    fn back_to_title(&mut self) {
        self.star_manager.borrow_mut().set_stop(false);
        self.state = AppState::Title;
        self.count = 0;
    }
}

impl<R: RendererTrait> AppTrait<R> for GalanguaApp {
    fn on_key(&mut self, keycode: Keycode, down: bool) {
        self.pad.on_key(keycode, down);
    }

    fn on_joystick_axis(&mut self, axis_index: u8, dir: i8) {
        self.pad.on_joystick_axis(axis_index, dir);
    }

    fn on_joystick_button(&mut self, button_index: u8, down: bool) {
        self.pad.on_joystick_button(button_index, down);
    }

    fn init(&mut self, renderer: &mut R) -> Result<(), String>
        where R: RendererTrait
    {
        renderer.load_textures("assets", &vec!["chr.png", "font.png"])?;

        let sprite_sheet = load_sprite_sheet("assets/chr.json").expect("sprite sheet");
        renderer.set_sprite_sheet(sprite_sheet);

        Ok(())
    }

    fn update(&mut self) -> bool {
        self.pad.update();

        if self.pad.is_trigger(PadBit::CANCEL) {
            if self.state != AppState::Title {
                self.back_to_title();
            } else {
                return false;
            }
        }

        if self.pad.is_trigger(PadBit::START) {
            self.paused = !self.paused;
        }
        if self.paused {
            return true;
        }

        self.star_manager.borrow_mut().update();

        match self.state {
            AppState::Title => {
                self.count = self.count.wrapping_add(1);

                if self.pad.is_trigger(PadBit::A) {
                    self.game_manager.restart();
                    self.state = AppState::Game;
                    self.frame_count = 0;
                }
            }
            AppState::Game => {
                self.frame_count += 1;
                self.game_manager.update(&self.pad);
                if self.game_manager.is_finished() {
                    self.back_to_title();
                }
            }
        }
        true
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
                renderer.draw_str("font", 10 * 8, 8 * 8, "GALANGUA")?;

                if self.count & 32 == 0 {
                    renderer.draw_str("font", 2 * 8, 25 * 8, "PRESS SPACE KEY TO START")?;
                }
            }
            AppState::Game => {
                self.game_manager.draw(renderer)?;
            }
        }

        renderer.set_texture_color_mod("font", 255, 0, 0);
        if (self.frame_count & 31) < 16 || self.state != AppState::Game {
            renderer.draw_str("font", 8 * 2, 8 * 0, "1UP")?;
        }
        renderer.draw_str("font", 8 * 9, 8 * 0, "HIGH SCORE")?;
        renderer.set_texture_color_mod("font", 255, 255, 255);
        renderer.draw_str("font", 8 * 0, 8 * 1,
                          &format!("{:6}0", self.game_manager.score() / 10))?;
        renderer.draw_str("font", 8 * 10, 8 * 1,
                          &format!("{:6}0", self.game_manager.high_score() / 10))?;

        renderer.set_texture_color_mod("font", 128, 128, 128);
        renderer.draw_str("font", 8 * 23, 8 * 0, &format!("FPS{:2}", self.fps_calc.fps()))?;

        renderer.present();

        self.fps_calc.update();

        Ok(())
    }
}

use super::game::effect::StarManager;
use super::game::game_manager::GameManager;
use super::game::game_manager::Params as GameManagerParams;

use crate::framework::{AppTrait, RendererTrait, VKey};
use crate::util::fps_calc::{FpsCalc, TimerTrait};
use crate::util::pad::{Pad, PadBit};

#[derive(PartialEq)]
enum AppState {
    Title,
    Game,
}

pub struct GalanguaApp<T: TimerTrait> {
    state: AppState,
    count: u32,
    pad: Pad,
    fps_calc: FpsCalc<T>,
    game_manager: GameManager,
    star_manager: StarManager,
    frame_count: u32,

    #[cfg(debug_assertions)]
    paused: bool,
}

impl<T: TimerTrait> GalanguaApp<T> {
    pub fn new(timer: T) -> Self {
        let star_manager = StarManager::new();
        Self {
            state: AppState::Title,
            count: 0,
            pad: Pad::new(),
            fps_calc: FpsCalc::new(timer),
            game_manager: GameManager::new(),
            star_manager,
            frame_count: 0,

            #[cfg(debug_assertions)]
            paused: false,
        }
    }

    fn update_main(&mut self) -> bool {
        if self.pad.is_trigger(PadBit::CANCEL) {
            if self.state != AppState::Title {
                self.back_to_title();
            } else {
                return false;
            }
        }

        #[cfg(debug_assertions)]
        {
            if self.pad.is_trigger(PadBit::START) {
                self.paused = !self.paused;
            }
            if self.paused {
                return true;
            }
        }

        self.star_manager.update();

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
                let mut params = GameManagerParams {
                    star_manager: &mut self.star_manager,
                    pad: &self.pad,
                };
                self.game_manager.update(&mut params);
                if self.game_manager.is_finished() {
                    self.back_to_title();
                }
            }
        }
        true
    }

    fn draw_main<R>(&mut self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        self.star_manager.draw(renderer)?;
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

        Ok(())
    }

    fn back_to_title(&mut self) {
        self.star_manager.set_stop(false);
        self.state = AppState::Title;
        self.count = 0;
    }
}

impl<R: RendererTrait, T: TimerTrait> AppTrait<R> for GalanguaApp<T> {
    fn on_key(&mut self, vkey: VKey, down: bool) {
        self.pad.on_key(vkey, down);
    }

    fn on_joystick_axis(&mut self, axis_index: u8, dir: i8) {
        self.pad.on_joystick_axis(axis_index, dir);
    }

    fn on_joystick_button(&mut self, button_index: u8, down: bool) {
        self.pad.on_joystick_button(button_index, down);
    }

    fn init(&mut self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        renderer.load_textures("assets", &["chr.png", "font.png"])?;
        renderer.load_sprite_sheet("assets/chr.json")?;
        Ok(())
    }

    fn update(&mut self) -> bool {
        self.pad.update();
        let result = self.update_main();
        result
    }

    fn draw(&mut self, renderer: &mut R) -> Result<(), String>
    where
        R: RendererTrait,
    {
        renderer.set_draw_color(0, 0, 0);
        renderer.clear();

        self.draw_main(renderer)?;

        renderer.present();

        self.fps_calc.update();

        Ok(())
    }
}

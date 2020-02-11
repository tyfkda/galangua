extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

pub trait App {
    fn init(&mut self, canvas: &mut WindowCanvas) -> Result<(), String>;
    fn update(&mut self);
    fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String>;

    fn on_key_down(&mut self, keycode: Keycode);
    fn on_key_up(&mut self, keycode: Keycode);
}

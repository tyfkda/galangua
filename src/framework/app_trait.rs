use sdl2::keyboard::Keycode;

use super::RendererTrait;

pub trait App<Renderer: RendererTrait> {
    fn init(&mut self, renderer: &mut Renderer) -> Result<(), String>;
    fn update(&mut self);
    fn draw(&mut self, renderer: &mut Renderer) -> Result<(), String>;

    fn on_key_down(&mut self, keycode: Keycode);
    fn on_key_up(&mut self, keycode: Keycode);
}

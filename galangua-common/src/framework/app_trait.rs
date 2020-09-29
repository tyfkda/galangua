use crate::framework::{RendererTrait, VKey};

pub trait AppTrait<R: RendererTrait> {
    fn init(&mut self, renderer: &mut R);
    fn update(&mut self) -> bool;
    fn draw(&mut self, renderer: &mut R);

    fn on_key(&mut self, keycode: VKey, down: bool);
    fn on_joystick_axis(&mut self, axis_index: u8, dir: i8);
    fn on_joystick_button(&mut self, button_index: u8, down: bool);
}

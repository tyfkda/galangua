mod app;
mod framework;
mod sdl;
mod util;

use sdl2::keyboard::Keycode;

use crate::app::consts;
use crate::app::GalanguaApp;
use crate::framework::VKey;
use crate::sdl::SdlAppFramework;

pub fn main() -> Result<(), String> {
    let app = GalanguaApp::new();
    let mut framework = SdlAppFramework::new(Box::new(app), map_key)?;
    framework.run("Galangua",
                  consts::WIDTH as u32, consts::HEIGHT as u32, 3)
}

fn map_key(keycode: Keycode) -> Option<VKey> {
    match keycode {
        Keycode::Space => Some(VKey::Space),
        Keycode::Return => Some(VKey::Return),
        Keycode::Escape => Some(VKey::Escape),
        Keycode::Left => Some(VKey::Left),
        Keycode::Right => Some(VKey::Right),
        Keycode::Up => Some(VKey::Up),
        Keycode::Down => Some(VKey::Down),
        _ => None,
    }
}

extern crate sdl2;

mod app;
mod framework;
mod util;

use self::app::{GaragaApp};
use self::framework::{SdlAppFramework};

pub fn main() -> Result<(), String> {
    let app = GaragaApp::new();
    let mut framework = SdlAppFramework::new("Garaga", 240 * 2, 320 * 2, Box::new(app))?;
    framework.run()
}

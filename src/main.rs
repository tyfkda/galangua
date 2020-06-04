mod app;
mod framework;
mod sdl;
mod util;

use crate::app::consts;
use crate::app::GalanguaApp;
use crate::sdl::SdlAppFramework;

pub fn main() -> Result<(), String> {
    let app = GalanguaApp::new();
    let mut framework = SdlAppFramework::new(Box::new(app))?;
    framework.run("Galangua",
                  consts::WIDTH as u32, consts::HEIGHT as u32, 3)
}

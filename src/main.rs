mod app;
mod framework;
mod util;

use crate::app::GaragaApp;
use crate::app::consts;
use crate::framework::SdlAppFramework;

pub fn main() -> Result<(), String> {
    let app = GaragaApp::new();
    let mut framework = SdlAppFramework::new(Box::new(app))?;
    framework.run("Garaga",
                  (consts::WIDTH as u32) * 2,
                  (consts::HEIGHT as u32) * 2)
}

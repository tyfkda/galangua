mod app;
mod framework;
mod util;

use self::app::GaragaApp;
use self::app::consts;
use self::framework::SdlAppFramework;

pub fn main() -> Result<(), String> {
    let app = GaragaApp::new();
    let mut framework = SdlAppFramework::new(Box::new(app))?;
    framework.run("Garaga",
                  (consts::WIDTH as u32) * 2,
                  (consts::HEIGHT as u32) * 2)
}

#[macro_use]
extern crate lazy_static;

mod app;
mod framework;
mod util;

use self::app::GaragaApp;

pub fn main() -> Result<(), String> {
    GaragaApp::generate_and_run()
}

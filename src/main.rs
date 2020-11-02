mod sdl;
mod std_system;
mod std_timer;

use counted_array::counted_array;
use lazy_static::lazy_static;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

use galangua_common::app::consts::*;
use galangua_common::framework::VKey;
use galangua_core::app::GalanguaApp;

use crate::sdl::SdlAppFramework;
use crate::sdl::SdlAudio;
use crate::std_system::StdSystem;
use crate::std_timer::StdTimer;

const APP_NAME: &str = "Galangua";

pub fn main() -> Result<(), String> {
    let matches = clap::App::new(APP_NAME)
        .version("0.9.1")
        .about("2D shoot'em up game, writen in Rust.
  Move the fighter : Arrow keys (left or right)
  Shoot a bullet   : Space bar
  Quit the app     : Escape key")
        .arg(clap::Arg::with_name("full")
             .help("Use fullscreen")
             .short("f")
             .long("fullscreen"))
        .arg(clap::Arg::with_name("scale")
             .help("Specify window scale (default: 3)")
             .short("s")
             .long("scale")
             .takes_value(true))
        .get_matches();

    let fullscreen = matches.is_present("full");
    let scale = if let Some(scale) = matches.value_of("scale") {
        String::from(scale).parse().unwrap()
    } else {
        3
    };

    let timer = StdTimer::new();
    let audio = SdlAudio::new(CHANNEL_COUNT, BASE_VOLUME);
    let system = StdSystem::new(audio);
    let app = GalanguaApp::new(timer, system);
    let mut framework = SdlAppFramework::new(app, map_key)?;
    framework.run(APP_NAME,
                  WIDTH as u32, HEIGHT as u32, scale, fullscreen)
}

counted_array!(const KEY_MAP_TABLE: [(Keycode, VKey); _] = [
    (Keycode::Space,  VKey::Space),
    (Keycode::Return, VKey::Return),
    (Keycode::Escape, VKey::Escape),
    (Keycode::Left,   VKey::Left),
    (Keycode::Right,  VKey::Right),
    (Keycode::Up,     VKey::Up),
    (Keycode::Down,   VKey::Down),

    (Keycode::A, VKey::A), (Keycode::B, VKey::B), (Keycode::C, VKey::C), (Keycode::D, VKey::D),
    (Keycode::E, VKey::E), (Keycode::F, VKey::F), (Keycode::G, VKey::G), (Keycode::H, VKey::H),
    (Keycode::I, VKey::I), (Keycode::J, VKey::J), (Keycode::K, VKey::K), (Keycode::L, VKey::L),
    (Keycode::M, VKey::M), (Keycode::N, VKey::N), (Keycode::O, VKey::O), (Keycode::P, VKey::P),
    (Keycode::Q, VKey::Q), (Keycode::R, VKey::R), (Keycode::S, VKey::S), (Keycode::T, VKey::T),
    (Keycode::U, VKey::U), (Keycode::V, VKey::V), (Keycode::W, VKey::W), (Keycode::X, VKey::X),
    (Keycode::Y, VKey::Y), (Keycode::Z, VKey::Z),

    (Keycode::Num0, VKey::Num0), (Keycode::Num1, VKey::Num1), (Keycode::Num2, VKey::Num2),
    (Keycode::Num3, VKey::Num3), (Keycode::Num4, VKey::Num4), (Keycode::Num5, VKey::Num5),
    (Keycode::Num6, VKey::Num6), (Keycode::Num7, VKey::Num7), (Keycode::Num8, VKey::Num8),
    (Keycode::Num9, VKey::Num9),
]);

lazy_static! {
    static ref KEY_MAP: HashMap<Keycode, VKey> = {
        let mut m = HashMap::new();
        for &(keycode, vkey) in KEY_MAP_TABLE.iter() {
            m.insert(keycode, vkey);
        }
        m
    };
}

fn map_key(keycode: Keycode) -> Option<VKey> {
    KEY_MAP.get(&keycode).map(|x| *x)
}

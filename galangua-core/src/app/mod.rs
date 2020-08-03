pub mod consts;
mod galangua_app;
mod game;
mod util;

pub use self::galangua_app::GalanguaApp;

#[cfg(debug_assertions)]
mod debug;

extern crate galangua_core;
extern crate web_sys;

mod utils;
mod wasm_app_framework;
mod wasm_renderer;
mod wasm_system;
mod wasm_timer;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

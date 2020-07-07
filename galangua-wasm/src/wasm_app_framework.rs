use wasm_bindgen::prelude::*;

use galangua_core::app::GalanguaApp;
use galangua_core::framework::{AppTrait, VKey};

use super::wasm_renderer::WasmRenderer;

#[wasm_bindgen]
pub struct WasmAppFramework {
    app: Box<dyn AppTrait<WasmRenderer>>,
    renderer: WasmRenderer,
}

#[wasm_bindgen]
impl WasmAppFramework {
    pub fn new(mut renderer: WasmRenderer) -> Self {
        web_sys::console::log_1(&"WasmAppFramework#new".into());

        let mut app = Box::new(GalanguaApp::new());

        app.init(&mut renderer).expect("app.init failed");

        Self {
            app,
            renderer,
        }
    }

    pub fn on_key(&mut self, key_code: &str, down: bool) {
        if let Some(vkey) = to_vkey(key_code) {
            self.app.on_key(vkey, down);
        }
    }

    pub fn update(&mut self) {
        self.app.update();
    }

    pub fn draw(&mut self) {
        self.app.draw(&mut self.renderer)
            .unwrap_or_else(|e| {
                web_sys::console::error_1(&format!("err: {:?}", e.to_string()).into());
            })
    }
}

fn to_vkey(key_code: &str) -> Option<VKey> {
    match key_code {
        "Space" => Some(VKey::Space),
        "Enter" => Some(VKey::Return),
        "Escape" => Some(VKey::Escape),
        "ArrowLeft" => Some(VKey::Left),
        "ArrowRight" => Some(VKey::Right),
        "ArrowUp" => Some(VKey::Up),
        "ArrowDown" => Some(VKey::Down),
        _ => None,
    }
}

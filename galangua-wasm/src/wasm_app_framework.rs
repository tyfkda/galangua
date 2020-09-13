use wasm_bindgen::prelude::*;

use galangua_core::app::GalanguaApp;
use galangua_core::framework::{AppTrait, VKey};

use super::wasm_renderer::WasmRenderer;
use super::wasm_system::WasmSystem;
use super::wasm_timer::WasmTimer;

#[wasm_bindgen]
pub struct WasmAppFramework {
    app: Box<dyn AppTrait<WasmRenderer>>,
    renderer: WasmRenderer,
}

#[wasm_bindgen]
impl WasmAppFramework {
    pub fn new(
        mut renderer: WasmRenderer, get_now_fn: js_sys::Function,
        get_item_fn: js_sys::Function, set_item_fn: js_sys::Function,
    ) -> Self {
        web_sys::console::log_1(&"WasmAppFramework#new".into());

        let timer = WasmTimer::new(move || {
            let this = JsValue::NULL;
            if let Ok(v) = get_now_fn.call0(&this) {
                if let Some(t) = v.as_f64() {
                    return t;
                }
            }
            0.0
        });
        let system = WasmSystem::new(
            move |key| {
                let this = JsValue::NULL;
                get_item_fn.call1(&this, &JsValue::from(key)).ok()
            },
            move |key, value| {
                let this = JsValue::NULL;
                set_item_fn.call2(&this, &JsValue::from(key), &JsValue::from(value)).unwrap();
            },
        );
        let mut app = Box::new(GalanguaApp::new(timer, system));

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

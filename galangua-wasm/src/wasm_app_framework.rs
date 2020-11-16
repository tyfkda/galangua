use wasm_bindgen::prelude::*;

use galangua_common::framework::{AppTrait, VKey};
use galangua_ecs::app::GalanguaEcsApp;

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
        mut renderer: WasmRenderer,
        is_touch_device: bool,
        get_now_fn: js_sys::Function,
        get_item_fn: js_sys::Function, set_item_fn: js_sys::Function,
    ) -> Self {
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
            is_touch_device,
        );
        let mut app = GalanguaEcsApp::new(timer, system);

        app.init(&mut renderer);

        Self {
            app: Box::new(app),
            renderer,
        }
    }

    pub fn on_key(&mut self, key_code: &str, down: bool) {
        if let Some(vkey) = to_vkey(key_code) {
            self.app.on_key(vkey, down);
        }
    }

    pub fn on_touch(&mut self, num: i32, down: bool) {
        match num {
            -1 | 0 | 1 => {
                let keys = [(-1, VKey::Left), (1, VKey::Right)];
                for i in 0..keys.len() {
                    let d = down && keys[i].0 == num;
                    self.app.on_key(keys[i].1, d);
                }
            }
            100 => self.app.on_key(VKey::Space, down),
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.app.update();
    }

    pub fn draw(&mut self) {
        self.app.draw(&mut self.renderer);
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

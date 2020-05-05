use wasm_bindgen::prelude::*;

use galangua_core::framework::SystemTrait;

#[wasm_bindgen]
extern "C" {
    fn play_se(channel: u32, filename: &str);
}

pub struct WasmSystem<F, G>
where
    F: Fn(&str) -> Option<JsValue>,
    G: Fn(&str, JsValue),
{
    get_item: F,
    set_item: G,
}

impl<F, G> WasmSystem<F, G>
where
    F: Fn(&str) -> Option<JsValue>,
    G: Fn(&str, JsValue),
{
    pub fn new(get_item: F, set_item: G) -> Self {
        WasmSystem {
            get_item,
            set_item,
        }
    }
}

impl<F, G> SystemTrait for WasmSystem<F, G>
where
    F: Fn(&str) -> Option<JsValue>,
    G: Fn(&str, JsValue),
{
    fn get_u32(&self, key: &str) -> Option<u32> {
        if let Some(value) = (self.get_item)(key) {
            if let Some(string) = value.as_string() {
                return string.parse().ok();
            }
        }
        None
    }

    fn set_u32(&mut self, key: &str, value: u32) {
        (self.set_item)(key, JsValue::from(value));
    }

    fn play_se(&mut self, channel: u32, filename: &str) {
        play_se(channel, filename);
    }
}

use wasm_bindgen::prelude::*;

use galangua_common::framework::SystemTrait;

#[wasm_bindgen]
extern "C" {
    fn play_se(channel: u32, filename: &str);
}

pub struct WasmSystem<
    F: Fn(&str) -> Option<JsValue>,
    G: Fn(&str, JsValue)
> {
    get_item: F,
    set_item: G,
    is_touch_device: bool,
}

impl<
    F: Fn(&str) -> Option<JsValue>,
    G: Fn(&str, JsValue)
> WasmSystem<F, G> {
    pub fn new(get_item: F, set_item: G, is_touch_device: bool) -> Self {
        WasmSystem {
            get_item,
            set_item,
            is_touch_device,
        }
    }
}

impl<
    F: Fn(&str) -> Option<JsValue>,
    G: Fn(&str, JsValue)
> SystemTrait for WasmSystem<F, G> {
    fn get_u32(&self, key: &str) -> Option<u32> {
        (self.get_item)(key)
            .map(|value| value.as_string()).flatten()
            .map(|string| string.parse().ok()).flatten()
    }

    fn set_u32(&mut self, key: &str, value: u32) {
        (self.set_item)(key, JsValue::from(value));
    }

    fn is_touch_device(&self) -> bool { self.is_touch_device }

    fn play_se(&mut self, channel: u32, filename: &str) {
        play_se(channel, filename);
    }
}

extern crate js_sys;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlCanvasElement, HtmlImageElement, Request, RequestInit, RequestMode, Response};

use galangua_core::framework::sprite_sheet::SpriteSheet;
use galangua_core::framework::types::Vec2I;
use galangua_core::framework::RendererTrait;

#[wasm_bindgen]
pub struct WasmRenderer {
    canvas: HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    images: Rc<RefCell<HashMap<String, HtmlImageElement>>>,
    sprite_sheet: Rc<RefCell<SpriteSheet>>,
}

#[wasm_bindgen]
impl WasmRenderer {
    pub fn new(canvas_id: &str) -> Self {
        web_sys::console::log_1(&format!("WasmRenderer#new, {}", canvas_id).into());

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        Self {
            canvas,
            context,
            images: Rc::new(RefCell::new(HashMap::new())),
            sprite_sheet: Rc::new(RefCell::new(SpriteSheet::new())),
        }
    }
}

impl RendererTrait for WasmRenderer {
    fn load_textures(&mut self, base_path: &str, filenames: &[&str]) {
        for &filename in filenames.iter() {
            let image = Rc::new(RefCell::new(HtmlImageElement::new().unwrap()));

            let path: String = format!("{}/{}", base_path, filename);
            let basename = String::from(Path::new(filename).file_stem().unwrap().to_str().unwrap());
            {
                let basename = basename.clone();
                let images = self.images.clone();
                let image_dup = image.clone();
                let closure = Closure::once_into_js(move |_event: JsValue| {
                    web_sys::console::log_1(&format!("Image loaded: {}", &basename).into());

                    image_dup.borrow_mut().set_onerror(None);
                    image_dup.borrow_mut().set_onload(None);

                    let image = Rc::try_unwrap(image_dup).unwrap().into_inner();
                    images.borrow_mut().insert(basename, image);
                });
                let cb = closure.as_ref().unchecked_ref();
                image.borrow_mut().set_onload(Some(cb));
            }
            {
                let basename = basename.clone();
                let closure = Closure::wrap(Box::new(move |_event: JsValue| {
                    web_sys::console::log_1(&format!("Image load failed: {}", &basename).into());
                }) as Box<dyn FnMut(JsValue)>);
                let cb = closure.as_ref().unchecked_ref();
                image.borrow_mut().set_onerror(Some(cb));
                closure.forget();
            }
            image.borrow_mut().set_src(&path);
        }
    }

    fn load_sprite_sheet(&mut self, filename: &str) {
        let filename = String::from(filename);
        let sprite_sheet = self.sprite_sheet.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match request(filename).await {
                Ok(text) => {
                    sprite_sheet.borrow_mut().load_sprite_sheet(&text);
                }
                Err(error) => {
                    web_sys::console::error_1(&format!("error: {}", &error).into());
                }
            }
        });
    }

    fn clear(&mut self) {
        self.context.fill_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64)
    }

    fn set_texture_color_mod(&mut self, _tex_name: &str, _r: u8, _g: u8, _b: u8) {}

    fn draw_str(&mut self, tex_name: &str, x: i32, y: i32, text: &str) {
        let image = self.images.borrow();
        if let Some(image) = image.get(tex_name) {
            let mut x = x as f64;
            let y = y as f64;
            let w = 8.0;
            let h = 8.0;
            self.context.set_fill_style(&JsValue::from("rgb(255,0,0)"));
            for c in text.chars() {
                let u: i32 = ((c as i32) - (' ' as i32)) % 16 * 8;
                let v: i32 = ((c as i32) - (' ' as i32)) / 16 * 8;
                self.context
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &image, u as f64, v as f64, w, h,
                        x, y, w, h)
                    .expect("draw_image_with... failed");
                x += w;
            }
        }
    }

    fn draw_sprite(&mut self, sprite_name: &str, pos: &Vec2I) {
        let sprite_sheet = self.sprite_sheet.borrow();
        let (sheet, tex_name) = sprite_sheet.get(sprite_name)
            .expect("No sprite_sheet");
        let image = self.images.borrow();
        if let Some(image) = image.get(tex_name) {
            let mut pos = *pos;
            if let Some(trimmed) = &sheet.trimmed {
                pos.x += trimmed.sprite_source_size.x;
                pos.y += trimmed.sprite_source_size.y;
            }

            self.context
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &image, sheet.frame.x as f64, sheet.frame.y as f64,
                    sheet.frame.w as f64, sheet.frame.h as f64,
                    pos.x as f64, pos.y as f64, sheet.frame.w as f64, sheet.frame.h as f64)
                .expect("draw_image_with... failed");
        }
    }

    fn draw_sprite_rot(&mut self, sprite_name: &str, pos: &Vec2I, angle: u8,
                       center: Option<&Vec2I>) {
        let sprite_sheet = self.sprite_sheet.borrow();
        let (sheet, tex_name) = sprite_sheet.get(sprite_name)
            .expect("No sprite_sheet");
        let image = self.images.borrow();
        if let Some(image) = image.get(tex_name) {
            let mut pos = *pos;
            if let Some(trimmed) = &sheet.trimmed {
                pos.x += trimmed.sprite_source_size.x;
                pos.y += trimmed.sprite_source_size.y;
            }
            let center = center.map_or_else(
                || Vec2I::new(sheet.frame.w as i32 / 2, sheet.frame.h as i32 / 2),
                |v| *v);

            self.context.save();
            self.context.translate((pos.x + center.x) as f64, (pos.y + center.y) as f64)
                .expect("translate failed");
            self.context.rotate((angle as f64) * (2.0 * std::f64::consts::PI / 256.0))
                .expect("rotate failed");
            self.context.translate(-center.x as f64, -center.y as f64)
                .expect("translate failed");
            self.context
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &image, sheet.frame.x as f64, sheet.frame.y as f64, sheet.frame.w as f64, sheet.frame.h as f64,
                    0.0, 0.0, sheet.frame.w as f64, sheet.frame.h as f64)
                .expect("draw_image_with... failed");
            self.context.restore();
        }
    }

    fn set_draw_color(&mut self, r: u8, g: u8, b: u8) {
        self.context.set_fill_style(&JsValue::from(format!("rgb({},{},{})", r, g, b)));
    }

    fn fill_rect(&mut self, dst: Option<[&Vec2I; 2]>) {
        if let Some(dst) = dst {
            self.context.fill_rect(dst[0].x as f64, dst[0].y as f64, dst[1].x as f64, dst[1].y as f64);
        } else {
            self.context.fill_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64);
        }
    }
}

async fn request(url: String) -> Result<String, String> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)
        .or_else(|_| Err(String::from("request init failed")))?;

    request
        .headers()
        .set("Accept", "text/plain")
        .or_else(|_| Err(String::from("request header error")))?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
        .expect("future failed");

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text().expect("text"))
        .await.expect("await")
        .as_string().unwrap();

    Ok(text)
}

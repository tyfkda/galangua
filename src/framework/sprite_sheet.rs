use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Debug)]
pub struct Trimmed {
    pub sprite_source_size: Rect,
    pub source_size: Size,
}

#[derive(Debug)]
pub struct SpriteSheet {
    pub texture: String,
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: Option<Trimmed>,
}

pub fn load_sprite_sheet(text: &str) -> HashMap<String, SpriteSheet> {
    let deserialized: Value = serde_json::from_str(text)
        .expect("illegal json");

    let mut m = HashMap::new();

    for (key, sheet) in deserialized["frames"].as_object().expect("frames") {
        let frame = &sheet["frame"];

        let rect = convert_rect(frame);
        let rotated = sheet["rotated"].as_bool().unwrap();
        let mut trimmed = None;
        if sheet["trimmed"].as_bool() == Some(true) {
            let sprite_source_size = convert_rect(&sheet["spriteSourceSize"]);
            let source_size = convert_size(&sheet["sourceSize"]);
            trimmed = Some(Trimmed { sprite_source_size, source_size });
        }

        let texture = deserialized["meta"]["image"].as_str().unwrap();
        m.insert(get_mainname(key),
                 SpriteSheet {
                     texture: get_mainname(texture),
                     frame: rect,
                     rotated,
                     trimmed,
                 });
    }

    m
}

fn convert_rect(value: &Value) -> Rect {
    Rect {
        x: value["x"].as_i64().unwrap() as i32,
        y: value["y"].as_i64().unwrap() as i32,
        w: value["w"].as_i64().unwrap() as u32,
        h: value["h"].as_i64().unwrap() as u32,
    }
}

fn convert_size(value: &Value) -> Size {
    Size { w: value["w"].as_i64().unwrap() as u32,
           h: value["h"].as_i64().unwrap() as u32 }
}

fn get_mainname(filename: &str) -> String {
    let re = Regex::new(r"^(.*)\.\w+").unwrap();
    if let Some(caps) = re.captures(filename) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        filename.to_string()
    }
}

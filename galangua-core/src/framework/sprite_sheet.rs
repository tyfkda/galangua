use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Sheet {
    pub texture_name: String,
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: Option<Trimmed>,
}

#[derive(Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

#[derive(Clone)]
pub struct Trimmed {
    pub sprite_source_size: Rect,
    pub source_size: Size,
}

pub fn load_sprite_sheet(text: &str) -> HashMap<String, Sheet> {
    let deserialized: Value = serde_json::from_str(text).expect("illegal json");

    let texture_name = get_mainname(
        deserialized["meta"]["image"].as_str().unwrap());

    let mut sheets = HashMap::new();
    for (key, frame) in deserialized["frames"].as_object().expect("frames") {
        sheets.insert(
            get_mainname(key),
            convert_sheet(frame, texture_name.clone()));
    }
    sheets
}

fn convert_sheet(sheet: &Value, texture_name: String) -> Sheet {
    let frame = convert_rect(&sheet["frame"]);
    let rotated = sheet["rotated"].as_bool().unwrap();
    let trimmed = if sheet["trimmed"].as_bool() == Some(true) {
        let sprite_source_size = convert_rect(&sheet["spriteSourceSize"]);
        let source_size = convert_size(&sheet["sourceSize"]);
        Some(Trimmed { sprite_source_size, source_size })
    } else {
        None
    };

    Sheet {
        texture_name,
        frame,
        rotated,
        trimmed,
    }
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

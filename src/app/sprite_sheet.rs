use regex::Regex;
use serde_json::Value;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use super::super::framework::sprite_sheet::{SpriteSheet, Size, Rect, Trimmed};

pub fn load_sprite_sheet(filename: &str) -> Result<HashMap<String, SpriteSheet>, String> {
    let file = File::open(filename).expect(&format!("file not found: {}", filename));
    let reader = BufReader::new(file);

    let deserialized: Value = serde_json::from_reader(reader).expect(&format!("illegal json: {}", filename));

    let mut m = HashMap::new();

    for (key, sheet) in deserialized["frames"].as_object().expect("frames") {
        let frame = &sheet["frame"];

        let rect = convert_rect(frame);
        let rotated = sheet["rotated"].as_bool().unwrap();
        let mut trimmed = None;
        if sheet["trimmed"].as_bool() == Some(true) {
            let sprite_source_size = convert_rect(&sheet["spriteSourceSize"]);
            let source_size = convert_size(&sheet["sourceSize"]);
            trimmed = Some(Trimmed {sprite_source_size,
                                    source_size});
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

    Ok(m)
}

fn convert_rect(value: &Value) -> Rect {
    Rect {x: value["x"].as_i64().unwrap() as i32,
          y: value["y"].as_i64().unwrap() as i32,
          w: value["w"].as_i64().unwrap() as u32,
          h: value["h"].as_i64().unwrap() as u32}
}

fn convert_size(value: &Value) -> Size {
    Size {w: value["w"].as_i64().unwrap() as u32,
          h: value["h"].as_i64().unwrap() as u32}
}

fn get_mainname(filename: &str) -> String {
    let re = Regex::new(r"^(.*)\.\w+").unwrap();
    if let Some(caps) = re.captures(filename) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        filename.to_string()
    }
}

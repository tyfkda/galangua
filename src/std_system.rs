use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use super::sdl::SdlAudio;

use galangua_common::framework::SystemTrait;

const SAVE_FILE_NAME: &str = ".savedata.json";

pub struct StdSystem {
    map: HashMap<String, Value>,
    audio: SdlAudio,
}

impl StdSystem {
    pub fn new(audio: SdlAudio) -> Self {
        StdSystem {
            map: load_map(SAVE_FILE_NAME),
            audio,
        }
    }
}

impl SystemTrait for StdSystem {
    fn get_u32(&self, key: &str) -> Option<u32> {
        if let Some(Value::Number(num)) = self.map.get(key) {
            return Some(num.as_u64().unwrap() as u32);
        }
        None
    }

    fn set_u32(&mut self, key: &str, value: u32) {
        self.map.insert(String::from(key), Value::Number(serde_json::Number::from(value)));
        save_map(SAVE_FILE_NAME, &self.map);
    }

    fn play_se(&mut self, channel: u32, filename: &str) {
        self.audio.play_se(channel, filename);
    }
}

fn load_map(filename: &str) -> HashMap<String, Value> {
    if std::path::Path::new(filename).exists() {
        match std::fs::read_to_string(filename) {
            Ok(text) => match serde_json::from_str::<HashMap<String, Value>>(&text) {
                Ok(deserialized) => {
                    return deserialized;
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            },
            Err(err) => {
                eprintln!("{}", err);
            }
        };
    }
    HashMap::new()
}

fn save_map(filename: &str, map: &HashMap<String, Value>) {
    match serde_json::to_string(map) {
        Ok(serialized) => {
            let mut f = File::create(filename).expect("Unable to create file");
            f.write_all(serialized.as_bytes()).expect("Unable to write data");
        }
        Err(message) => {
            eprintln!("{}", message);
        }
    }
}

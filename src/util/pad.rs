use sdl2::keyboard::Keycode;

pub const PAD_L: u8 = 1 << 0;
pub const PAD_R: u8 = 1 << 1;
pub const PAD_A: u8 = 1 << 2;

pub struct Pad {
    pad: u8,
    trg: u8,
    last_pad: u8,
}

impl Pad {
    pub fn new() -> Pad {
        Pad {
            pad: 0,
            trg: 0,
            last_pad: 0,
        }
    }

    pub fn update(&mut self) {
        self.trg = self.pad & !self.last_pad;
        self.last_pad = self.pad;
    }

    pub fn on_key_down(&mut self, keycode: Keycode) {
        self.pad = self.pad | get_key_bit(keycode);
    }

    pub fn on_key_up(&mut self, keycode: Keycode) {
        self.pad = self.pad & !get_key_bit(keycode);
    }

    pub fn is_pressed(&self, btn: u8) -> bool {
        self.pad & btn != 0
    }

    pub fn is_trigger(&self, btn: u8) -> bool {
        self.trg & btn != 0
    }
}

fn get_key_bit(key: Keycode) -> u8 {
    return match key {
        Keycode::Left => PAD_L,
        Keycode::Right => PAD_R,
        Keycode::Space => PAD_A,
        _ => 0,
    }
}

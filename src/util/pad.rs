use sdl2::keyboard::Keycode;

pub const PAD_L: u8 = 1 << 0;
pub const PAD_R: u8 = 1 << 1;
pub const PAD_A: u8 = 1 << 2;
pub const PAD_START: u8 = 1 << 7;

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
    match key {
        Keycode::Left => PAD_L,
        Keycode::Right => PAD_R,
        Keycode::Space => PAD_A,
        Keycode::Return => PAD_START,
        _ => 0,
    }
}

#[test]
fn test_trigger() {
    let mut pad = Pad::new();
    pad.on_key_down(Keycode::Space);
    pad.update();

    assert_eq!(true, pad.is_pressed(PAD_A));
    assert_eq!(true, pad.is_trigger(PAD_A));

    pad.update();
    assert_eq!(true, pad.is_pressed(PAD_A));
    assert_eq!(false, pad.is_trigger(PAD_A));
}

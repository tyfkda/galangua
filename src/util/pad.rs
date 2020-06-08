use sdl2::keyboard::Keycode;

pub const PAD_L: u8 = 1 << 0;
pub const PAD_R: u8 = 1 << 1;
pub const PAD_U: u8 = 1 << 2;
pub const PAD_D: u8 = 1 << 3;
pub const PAD_A: u8 = 1 << 4;
pub const PAD_B: u8 = 1 << 5;
pub const PAD_CANCEL: u8 = 1 << 6;
pub const PAD_START: u8 = 1 << 7;

pub struct Pad {
    pad: u8,
    trg: u8,
    last_pad: u8,
    key: u8,
    joy: u8,
}

impl Pad {
    pub fn new() -> Self {
        Self {
            pad: 0,
            trg: 0,
            last_pad: 0,
            key: 0,
            joy: 0,
        }
    }

    pub fn update(&mut self) {
        self.pad = self.key | self.joy;
        self.trg = self.pad & !self.last_pad;
        self.last_pad = self.pad;
    }

    pub fn is_pressed(&self, btn: u8) -> bool {
        self.pad & btn != 0
    }

    pub fn is_trigger(&self, btn: u8) -> bool {
        self.trg & btn != 0
    }

    pub fn on_key(&mut self, keycode: Keycode, down: bool) {
        let bit = get_key_bit(keycode);
        if down {
            self.key |= bit;
        } else {
            self.key &= !bit;
        }
    }

    pub fn on_joystick_axis(&mut self, axis_index: u8, dir: i8) {
        match axis_index {
            0 => {
                let lr = if dir < 0 { PAD_L } else if dir > 0 { PAD_R } else { 0 };
                self.joy = (self.joy & !(PAD_L | PAD_R)) | lr;
            }
            1 => {
                let ud = if dir < 0 { PAD_U } else if dir > 0 { PAD_D } else { 0 };
                self.joy = (self.joy & !(PAD_U | PAD_D)) | ud;
            }
            _ => {}
        }
    }

    pub fn on_joystick_button(&mut self, button_index: u8, down: bool) {
        let bit;
        match button_index {
            0 => bit = PAD_A,
            1 => bit = PAD_B,
            _ => { return; }
        }
        if down {
            self.joy |= bit;
        } else {
            self.joy &= !bit;
        }
    }
}

fn get_key_bit(key: Keycode) -> u8 {
    match key {
        Keycode::Left => PAD_L,
        Keycode::Right => PAD_R,
        Keycode::Space => PAD_A,
        Keycode::Escape => PAD_CANCEL,
        Keycode::Return => PAD_START,
        _ => 0,
    }
}

#[test]
fn test_trigger() {
    let mut pad = Pad::new();
    pad.on_key(Keycode::Space, true);
    pad.update();

    assert_eq!(true, pad.is_pressed(PAD_A));
    assert_eq!(true, pad.is_trigger(PAD_A));

    pad.update();
    assert_eq!(true, pad.is_pressed(PAD_A));
    assert_eq!(false, pad.is_trigger(PAD_A));
}

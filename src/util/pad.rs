use bitflags::bitflags;

use crate::framework::VKey;

bitflags! {
    pub struct PadBit: u32 {
        const L      = 0b00000001;
        const R      = 0b00000010;
        const U      = 0b00000100;
        const D      = 0b00001000;
        const A      = 0b00010000;
        const B      = 0b00100000;
        const CANCEL = 0b01000000;
        const START  = 0b10000000;
    }
}

pub struct Pad {
    pad: PadBit,
    trg: PadBit,
    last_pad: PadBit,
    key: PadBit,
    joy: PadBit,
}

impl Pad {
    pub fn new() -> Self {
        let empty = PadBit::empty();
        Self {
            pad: empty,
            trg: empty,
            last_pad: empty,
            key: empty,
            joy: empty,
        }
    }

    pub fn update(&mut self) {
        self.pad = self.key | self.joy;
        self.trg = self.pad & !self.last_pad;
        self.last_pad = self.pad;
    }

    pub fn is_pressed(&self, btn: PadBit) -> bool {
        self.pad.contains(btn)
    }

    pub fn is_trigger(&self, btn: PadBit) -> bool {
        self.trg.contains(btn)
    }

    pub fn on_key(&mut self, keycode: VKey, down: bool) {
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
                let lr = if dir < 0 { PadBit::L }
                    else if dir > 0 { PadBit::R }
                    else { PadBit::empty() };
                self.joy = (self.joy & !(PadBit::L | PadBit::R)) | lr;
            }
            1 => {
                let ud = if dir < 0 { PadBit::U }
                    else if dir > 0 { PadBit::D }
                    else { PadBit::empty() };
                self.joy = (self.joy & !(PadBit::U | PadBit::D)) | ud;
            }
            _ => {}
        }
    }

    pub fn on_joystick_button(&mut self, button_index: u8, down: bool) {
        let bit;
        match button_index {
            0 => bit = PadBit::A,
            1 => bit = PadBit::B,
            _ => { return; }
        }
        if down {
            self.joy |= bit;
        } else {
            self.joy &= !bit;
        }
    }
}

fn get_key_bit(key: VKey) -> PadBit {
    match key {
        VKey::Left => PadBit::L,
        VKey::Right => PadBit::R,
        VKey::Space => PadBit::A,
        VKey::Escape => PadBit::CANCEL,
        VKey::Return => PadBit::START,
        _ => PadBit::empty(),
    }
}

#[test]
fn test_trigger() {
    let mut pad = Pad::new();
    pad.on_key(VKey::Space, true);
    pad.update();

    assert_eq!(true, pad.is_pressed(PadBit::A));
    assert_eq!(true, pad.is_trigger(PadBit::A));

    pad.update();
    assert_eq!(true, pad.is_pressed(PadBit::A));
    assert_eq!(false, pad.is_trigger(PadBit::A));
}

use crate::traits::*;

pub const KEY_UP: u8 = 2;
pub const KEY_DOWN: u8 = 3;
pub const KEY_LEFT: u8 = 1;
pub const KEY_RIGHT: u8 = 0;
pub const KEY_A: u8 = 4;
pub const KEY_B: u8 = 5;
pub const KEY_START: u8 = 7;
pub const KEY_SELECT: u8 = 6;

pub struct JoyPad {
    pub joypad_state: u8,
    pub input: u8,
}

impl Memory for JoyPad {
    fn read(&self, address: usize) -> u8 {
        match address {
            0xFF00 => self.get_joypad_state(),
            _ => panic!("Invalid JoyPad address"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0xFF00 => self.input = data,
            _ => panic!("Invalid JoyPad address"),
        }
    }
}

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad {
            joypad_state: 0xCF,
            input: 0x00,
        }
    }

    pub fn on_key_pressed(&mut self, key: u8) -> u8 {
        let mut interrupt_flag = 0;
        let previously_set = self.joypad_state.test_bit(key);
        self.joypad_state.reset_bit(key);
        let button = if key > 3 { true } else { false };
        if ((button && !self.input.test_bit(5)) || (!button && !self.input.test_bit(4)))
            && !previously_set
        {
            interrupt_flag |= 1 << 4;
        }

        interrupt_flag
    }

    pub fn on_key_released(&mut self, key: u8) {
        self.joypad_state.set_bit(key);
    }

    pub fn get_joypad_state(&self) -> u8 {
        let res = self.input ^ 0xFF; // TODO: move to constant
        if !res.test_bit(4) {
            res & ((self.joypad_state >> 4) | 0xF0)
        } else if !res.test_bit(5) {
            res & ((self.joypad_state & 0xF) | 0xF0)
        } else {
            res
        }
    }
}

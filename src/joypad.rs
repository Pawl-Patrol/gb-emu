use crate::traits::{Cartridge, SetBit, TestBit};

pub struct JoyPad {
    pub joypad_state: u8,
    pub input: u8,
    pub needs_interrupt: Option<u8>,
}

impl Cartridge for JoyPad {
    fn read(&self, address: usize) -> u8 {
        match address {
            0xFF00 => {
                println!(
                    "read from joypad: {:04X} = {:02X}",
                    address,
                    self.get_joypad_state()
                );
                self.get_joypad_state()
            }
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
            needs_interrupt: None,
        }
    }

    pub fn on_key_pressed(&mut self, key: u8) {
        let previously_set = self.joypad_state.test_bit(key);
        self.joypad_state.reset_bit(key);
        let button = if key > 3 { true } else { false };
        if ((button && !self.input.test_bit(5)) || (!button && !self.input.test_bit(4)))
            && !previously_set
        {
            self.needs_interrupt = Some(4);
        }
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

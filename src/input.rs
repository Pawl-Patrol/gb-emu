use crate::{canvas::*, constants::*, emulator::Emulator, traits::*};

impl Emulator {
    pub fn on_key_pressed(&mut self, key: u8) {
        let previously_set = self.joypad_state.test_bit(key);
        self.joypad_state.reset_bit(key);
        let button = if key > 3 { true } else { false };
        let key_request = self.mmu.read_byte(0xFF00);
        if ((button && !key_request.test_bit(5)) || (!button && !key_request.test_bit(4)))
            && !previously_set
        {
            self.request_interrupt(4);
        }
    }

    pub fn on_key_released(&mut self, key: u8) {
        self.joypad_state.set_bit(key);
    }

    pub fn get_joypad_state(&self) -> u8 {
        let res = self.mmu.read_byte(0xFF00) ^ 0xFF;
        if !res.test_bit(4) {
            res & ((self.joypad_state >> 4) | 0xF0)
        } else if !res.test_bit(5) {
            res & ((self.joypad_state & 0xF) | 0xF0)
        } else {
            res
        }
    }
}

use crate::{
    context::Context,
    traits::{SetBit, TestBit},
};

pub struct JoyPad {
    pub joypad_state: u8,
}

impl JoyPad {
    pub fn new() -> JoyPad {
        JoyPad { joypad_state: 0xcf }
    }

    pub fn on_key_pressed(&mut self, ctx: &mut Context, key: u8) {
        let previously_set = self.joypad_state.test_bit(key);
        self.joypad_state.reset_bit(key);
        let button = if key > 3 { true } else { false };
        let key_request = ctx.mmu.io[(0xFF00 - 0xFF00)]; // TODO: move to constant
        if ((button && !key_request.test_bit(5)) || (!button && !key_request.test_bit(4)))
            && !previously_set
        {
            ctx.cpu.request_interrupt(ctx, 4);
        }
    }

    pub fn on_key_released(&mut self, key: u8) {
        self.joypad_state.set_bit(key);
    }

    pub fn get_joypad_state(&self, ctx: &Context) -> u8 {
        let res = ctx.mmu.io[(0xFF00 - 0xFF00)] ^ 0xFF; // TODO: move to constant
        if !res.test_bit(4) {
            res & ((self.joypad_state >> 4) | 0xF0)
        } else if !res.test_bit(5) {
            res & ((self.joypad_state & 0xF) | 0xF0)
        } else {
            res
        }
    }
}

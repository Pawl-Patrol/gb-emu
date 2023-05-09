use crate::{canvas::*, constants::*, emulator::Emulator, traits::*};

impl Emulator {
    pub fn do_interrupts(&mut self) {
        if self.interrupts_enabled {
            let request = self.read_memory(INTERRUPT_FLAG);
            let enabled = self.read_memory(INTERRUPT_ENABLE);
            for i in 0..5 {
                if request.test_bit(i) && enabled.test_bit(i) {
                    self.service_interrupt(i);
                }
            }
        }
    }

    fn service_interrupt(&mut self, id: u8) {
        self.halted = false;
        self.interrupts_enabled = false;
        let mut request = self.read_memory(INTERRUPT_FLAG);
        request.reset_bit(id);
        self.write_memory(INTERRUPT_FLAG, request);

        self.push_stack(self.cpu.pc);

        match id {
            0 => self.cpu.pc = 0x40,
            1 => self.cpu.pc = 0x48,
            2 => self.cpu.pc = 0x50,
            3 => self.cpu.pc = 0x58,
            4 => self.cpu.pc = 0x60,
            _ => panic!("Invalid interrupt id"),
        }
    }

    pub fn request_interrupt(&mut self, id: u8) {
        let mut request = self.read_memory(INTERRUPT_FLAG);
        request.set_bit(id);
        self.write_memory(INTERRUPT_FLAG, request);
    }
}

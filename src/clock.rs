use crate::{constants::*, emulator::Emulator, traits::*};

impl Emulator {
    pub fn update_timers(&mut self, cycles: u16) {
        self.divider_counter = self.divider_counter.wrapping_add(cycles);
        if self.clock_enabled() {
            let threshold = self.get_clock_frequency();
            self.timer_counter += cycles as u32;
            while self.timer_counter >= threshold {
                self.timer_counter -= threshold;
                let (new_tima, overflow) = match self.read_memory(TIMA).checked_add(1) {
                    Some(new_tima) => (new_tima, false),
                    None => (self.read_memory(TMA), true),
                };
                self.write_memory(TIMA, new_tima);
                if overflow {
                    self.request_interrupt(2);
                }
            }
        }
    }

    fn get_clock_frequency(&self) -> u32 {
        match self.read_memory(TMC) & 0b0000_0011 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => panic!("Invalid timer frequency"),
        }
    }

    fn clock_enabled(&mut self) -> bool {
        self.read_memory(TMC).test_bit(2)
    }
}

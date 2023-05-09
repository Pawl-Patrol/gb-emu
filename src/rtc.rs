use crate::traits::{Cartridge, Register, TestBit};

pub struct RTC {
    pub timer_counter: u32,
    pub divider_counter: u16,
    pub tima: u8, // timer counter
    pub tma: u8,  // timer modulo
    pub tac: u8,  // timer control
    pub needs_interrupt: Option<u8>,
}

impl Cartridge for RTC {
    fn read(&self, address: usize) -> u8 {
        match address {
            0xFF04 => self.divider_counter.hi(),
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("Invalid RTC address"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0xFF04 => self.divider_counter = 0,
            0xFF05 => self.tima = data,
            0xFF06 => self.tma = data,
            0xFF07 => self.tac = data,
            _ => panic!("Invalid RTC address"),
        }
    }
}

impl RTC {
    pub fn new() -> RTC {
        RTC {
            timer_counter: 0,
            divider_counter: 0xAB00,
            tima: 0x00,
            tma: 0x00,
            tac: 0xF8,
            needs_interrupt: None,
        }
    }

    pub fn update_timers(&mut self, cycles: u16) {
        self.divider_counter = self.divider_counter.wrapping_add(cycles);
        if self.clock_enabled() {
            let threshold = self.get_clock_frequency();
            self.timer_counter += cycles as u32;
            while self.timer_counter >= threshold {
                self.timer_counter -= threshold;
                let (new_tima, overflow) = match self.tima.checked_add(1) {
                    Some(new_tima) => (new_tima, false),
                    None => (self.tma, true),
                };
                self.tima = new_tima;
                if overflow {
                    self.needs_interrupt = Some(2);
                }
            }
        }
    }

    fn get_clock_frequency(&self) -> u32 {
        match self.tac & 0b0000_0011 {
            0b00 => 4096,
            0b01 => 262144,
            0b10 => 65536,
            0b11 => 16384,
            _ => panic!("Invalid timer frequency"),
        }
    }

    fn clock_enabled(&mut self) -> bool {
        self.tac.test_bit(2)
    }
}

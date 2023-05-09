use crate::{constants::*, context::Context, traits::TestBit};

pub struct RTC {
    pub timer_counter: u32,
    pub divider_counter: u16,
}

impl RTC {
    pub fn new() -> RTC {
        RTC {
            timer_counter: 0,
            divider_counter: 0,
        }
    }

    pub fn update_timers(&mut self, ctx: &mut Context, cycles: u16) {
        self.divider_counter = self.divider_counter.wrapping_add(cycles);
        if self.clock_enabled(ctx) {
            let threshold = self.get_clock_frequency(ctx);
            self.timer_counter += cycles as u32;
            while self.timer_counter >= threshold {
                self.timer_counter -= threshold;
                let (new_tima, overflow) = match ctx.mmu.read_byte(ctx, TIMA).checked_add(1) {
                    Some(new_tima) => (new_tima, false),
                    None => (ctx.mmu.read_byte(ctx, TMA), true),
                };
                ctx.mmu.write_byte(ctx, TIMA, new_tima);
                if overflow {
                    ctx.cpu.request_interrupt(ctx, 2);
                }
            }
        }
    }

    fn get_clock_frequency(&self, ctx: &Context) -> u32 {
        match ctx.mmu.read_byte(ctx, TMC) & 0b0000_0011 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => panic!("Invalid timer frequency"),
        }
    }

    fn clock_enabled(&mut self, ctx: &Context) -> bool {
        ctx.mmu.read_byte(ctx, TMC).test_bit(2)
    }
}

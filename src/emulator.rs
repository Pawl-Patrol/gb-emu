use crate::cartridge::Cartridge;
use crate::constants::*;
use crate::cpu::CPU;
use crate::mmu::MMU;
use crate::traits::{Register, SetBit, TestBit};

pub struct Emulator {
    pub cpu: CPU,
    pub mmu: MMU,
    pub cartrige: Cartridge,
    pub screen: [[Color; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub current_rom_bank: usize,
    pub current_ram_bank: usize,
    pub rom_banking: bool,
    pub enable_ram: bool,
    pub timer_counter: u32,
    pub divider_counter: u32,
    pub interrupts_enabled: bool,
    pub scanline_counter: u32,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: CPU::new(),
            mmu: MMU::new(),
            cartrige: Cartridge::new(),
            screen: [[COLOR_WHITE; SCREEN_WIDTH]; SCREEN_HEIGHT],
            current_rom_bank: 1,
            current_ram_bank: 0,
            enable_ram: false,
            rom_banking: false,
            timer_counter: 0,
            divider_counter: 0,
            interrupts_enabled: false,
            scanline_counter: 0,
        }
    }

    fn update(&mut self) {
        let mut cycles_total: u32 = 0;
        while cycles_total < CYCLES_PER_FRAME {
            let cycles = self.execute_next_opcode();
            cycles_total += cycles;
            self.update_timers(cycles);
            self.update_graphics(cycles);
            self.do_interrupts();
        }
    }

    fn execute_next_opcode(&mut self) -> u32 {
        let opcode = self.read_memory(self.cpu.pc);
        self.cpu.pc += 1;
        return self.execute(opcode);
    }

    fn update_timers(&mut self, cycles: u32) {
        self.update_divider_register(cycles);
        if self.clock_enabled() {
            if let Some(new_timer_counter) = self.timer_counter.checked_sub(cycles) {
                self.timer_counter = new_timer_counter;
            } else {
                self.set_clock_frequency();
                if self.read_memory(TIMA) == 255 {
                    self.write_memory(TIMA, self.read_memory(TMA));
                    self.request_interrupt(2);
                } else {
                    self.write_memory(TIMA, self.read_memory(TIMA) + 1);
                }
            }
        }
    }

    fn get_clock_frequency(&self) -> u8 {
        self.read_memory(TMC) & 0b0000_0011
    }

    fn set_clock_frequency(&mut self) {
        let frequency = self.get_clock_frequency();
        match frequency {
            0 => self.timer_counter = TIMER_FREQ_0,
            1 => self.timer_counter = TIMER_FREQ_1,
            2 => self.timer_counter = TIMER_FREQ_2,
            3 => self.timer_counter = TIMER_FREQ_3,
            _ => panic!("Invalid timer frequency"),
        }
    }

    fn update_divider_register(&mut self, cycles: u32) {
        if let Some(new_divider_register) = self.divider_counter.checked_add(cycles) {
            self.divider_counter = new_divider_register;
        } else {
            self.divider_counter = 0;
            self.mmu.write_byte(DIV, self.mmu.read_byte(DIV) + 1);
        }
    }

    fn clock_enabled(&mut self) -> bool {
        self.read_memory(TMC).test_bit(2)
    }

    fn update_graphics(&mut self, cycles: u32) {
        self.set_lcd_status();

        if !self.lcd_enabled() {
            return;
        }

        if let Some(new_scanline_counter) = self.scanline_counter.checked_sub(cycles) {
            self.scanline_counter = new_scanline_counter;
        } else {
            self.scanline_counter = SCANLINE_CYCLES;
            let mut scanline = self.mmu.read_byte(SCANLINE);
            scanline += 1;
            self.mmu.write_byte(SCANLINE, scanline);

            if scanline == 144 {
                self.request_interrupt(0);
            } else if scanline > 153 {
                self.mmu.write_byte(SCANLINE, 0);
            } else if scanline < 144 {
                self.draw_scanline();
            }
        }
    }

    fn set_lcd_status(&mut self) {
        let mut status = self.read_memory(LCD_STATUS);

        if !self.lcd_enabled() {
            self.scanline_counter = SCANLINE_CYCLES;
            self.mmu.write_byte(SCANLINE, 0);
            status &= 0b1111_1100;
            status.set_bit(0);
            self.write_memory(LCD_STATUS, status);
            return;
        }

        let scanline = self.read_memory(SCANLINE);
        let current_mode = status & 0b0000_0011;
        let mut mode = 0;
        let mut request_interrupt = false;

        if scanline >= 144 {
            mode = 1;
            status.set_bit(0);
            status.reset_bit(1);
            request_interrupt = status.test_bit(4);
        } else {
            let mode2_bounds = SCANLINE_CYCLES - 80;
            let mode3_bounds = mode2_bounds - 172;

            if self.scanline_counter >= mode2_bounds {
                mode = 2;
                status.set_bit(1);
                status.reset_bit(0);
                request_interrupt = status.test_bit(5);
            } else if self.scanline_counter >= mode3_bounds {
                mode = 3;
                status.set_bit(1);
                status.set_bit(0);
            } else {
                mode = 0;
                status.reset_bit(1);
                status.reset_bit(0);
                request_interrupt = status.test_bit(3);
            }
        }

        if request_interrupt && mode != current_mode {
            self.request_interrupt(1);
        }

        if scanline == self.read_memory(LYC) {
            status.set_bit(2);
            if status.test_bit(6) {
                self.request_interrupt(1);
            }
        } else {
            status.reset_bit(2);
        }

        self.write_memory(LCD_STATUS, status);
    }

    fn lcd_enabled(&self) -> bool {
        self.read_memory(LCD_CONTROL).test_bit(7)
    }

    fn draw_scanline(&self) {
        todo!();
    }

    fn request_interrupt(&mut self, id: u8) {
        let mut request = self.read_memory(INTERRUPT_FLAG);
        request.set_bit(id);
        self.write_memory(INTERRUPT_FLAG, request);
    }

    fn do_interrupts(&mut self) {
        if self.interrupts_enabled {
            let request = self.read_memory(INTERRUPT_FLAG);
            let enabled = self.read_memory(INTERRUPT_ENABLE);
            if request > 0 {
                for i in 0..5 {
                    if request.test_bit(i) && enabled.test_bit(i) {
                        self.service_interrupt(i);
                    }
                }
            }
        }
    }

    fn service_interrupt(&mut self, id: u8) {
        self.interrupts_enabled = false;
        let mut request = self.read_memory(INTERRUPT_FLAG);
        request.reset_bit(id);
        self.write_memory(INTERRUPT_FLAG, request);

        self.push_stack(self.cpu.pc);

        match id {
            0 => self.cpu.pc = 0x40,
            1 => self.cpu.pc = 0x48,
            2 => self.cpu.pc = 0x50,
            4 => self.cpu.pc = 0x60,
            _ => panic!("Invalid interrupt id"),
        }
    }

    pub fn push_stack(&mut self, data: u16) {
        self.write_memory(self.cpu.sp - 1, (data >> 8) as u8);
        self.write_memory(self.cpu.sp - 2, (data & 0xFF) as u8);
        self.cpu.sp -= 2;
    }

    pub fn pop_stack(&mut self) -> u16 {
        let lo = self.read_memory(self.cpu.sp);
        let hi = self.read_memory(self.cpu.sp + 1);
        self.cpu.sp += 2;
        return u16::from_bytes(hi, lo);
    }

    pub fn read_memory(&self, address: u16) -> u8 {
        if address >= 0x4000 && address <= 0x7FFF {
            // rom memory bank
            let rom_address = address as usize - 0x4000;
            let bank_offset = self.current_rom_bank * ROM_BANK_SIZE;
            return self.cartrige.rom[rom_address + bank_offset];
        } else if address >= 0xA000 && address <= 0xBFFF {
            // ram memory bank
            let ram_address = address as usize - 0xA000;
            let bank_offset = self.current_ram_bank * RAM_BANK_SIZE;
            return self.cartrige.ram[ram_address + bank_offset];
        }
        // read memory
        return self.mmu.read_byte(address);
    }

    pub fn write_memory(&mut self, address: u16, data: u8) {
        if address < 0x8000 {
            self.handle_banking(address, data);
        } else if address >= 0xA000 && address < 0xC000 {
            if (self.enable_ram) {
                let ram_address = address as usize - 0xA000;
                let bank_offset = self.current_ram_bank * RAM_BANK_SIZE;
                self.cartrige.ram[ram_address + bank_offset] = data;
            }
            return;
        } else if address >= 0xFEA0 && address < 0xFEFF {
            return; // prohibited
        } else if address >= 0xE000 && address < 0xFE00 {
            // echo ram
            self.write_memory(address - 0x2000, data);
            return;
        } else if address == TMC {
            let frequency = self.get_clock_frequency();
            self.mmu.write_byte(TMC, data);
            let new_frequency = self.get_clock_frequency();
            if frequency != new_frequency {
                self.set_clock_frequency();
            }
            return;
        } else if address == DIV {
            self.divider_counter = 0;
            return;
        } else if address == SCANLINE {
            self.mmu.write_byte(SCANLINE, 0);
            return;
        } else if address == DMA {
            self.dma_transfer(data);
            return;
        }
        // write memory
        self.mmu.write_byte(address, data);
    }

    fn dma_transfer(&mut self, data: u8) {
        let address = (data as u16) << 8;
        for i in 0..0xA0 {
            let value = self.read_memory(address + i);
            self.write_memory(0xFE00 + i, value);
        }
    }

    fn handle_banking(&mut self, address: u16, data: u8) {
        if address < 0x2000 {
            if self.cartrige.bank_mode == MBC1 || self.cartrige.bank_mode == MBC2 {
                self.enable_ram_banking(address, data);
            }
        } else if address >= 0x200 && address < 0x4000 {
            if self.cartrige.bank_mode == MBC1 || self.cartrige.bank_mode == MBC2 {
                self.change_lo_ram_bank(data);
            }
        } else if address >= 0x4000 && address < 0x6000 {
            if self.cartrige.bank_mode == MBC1 {
                if self.rom_banking {
                    self.change_hi_rom_bank(data);
                } else {
                    self.change_ram_bank(data);
                }
            }
        } else if address >= 0x6000 && address < 0x8000 {
            if self.cartrige.bank_mode == MBC1 {
                self.change_rom_ram_mode(data);
            }
        }
    }

    fn enable_ram_banking(&mut self, address: u16, data: u8) {
        if self.cartrige.bank_mode == MBC2 && address.test_bit(4) {
            return;
        }

        let test_data = data & 0xF;
        if test_data == 0xA {
            self.enable_ram = true;
        } else if test_data == 0x0 {
            self.enable_ram = false;
        }
    }

    fn change_lo_ram_bank(&mut self, data: u8) {
        if self.cartrige.bank_mode == MBC1 {
            let lower5 = data & 0b0001_1111;
            self.current_rom_bank &= 0b1110_0000;
            self.current_rom_bank |= lower5 as usize;
        } else if self.cartrige.bank_mode == MBC2 {
            self.current_rom_bank = (data & 0b0000_1111) as usize;
        }

        if self.current_rom_bank == 0 {
            self.current_rom_bank = 1;
        }
    }

    fn change_hi_rom_bank(&mut self, data: u8) {
        self.current_rom_bank &= 0b0001_1111;
        self.current_rom_bank |= (data & 0b1110_0000) as usize;

        if self.current_rom_bank == 0 {
            self.current_rom_bank = 1;
        }
    }

    fn change_ram_bank(&mut self, data: u8) {
        self.current_ram_bank = (data & 0b0000_0011) as usize;
    }

    fn change_rom_ram_mode(&mut self, data: u8) {
        self.rom_banking = !data.test_bit(0);
        if self.rom_banking {
            self.current_ram_bank = 0;
        }
    }
}

use std::io::Write;

use crate::canvas::{COLOR_BLACK, COLOR_DARK_GRAY, COLOR_LIGHT_GRAY, COLOR_WHITE};
use crate::cartridge::{load_rom, Cartridge};
use crate::constants::*;
use crate::cpu::CPU;
use crate::mmu::MMU;
use crate::traits::{Register, SetBit, TestBit};

pub struct Emulator {
    pub cpu: CPU,
    pub mmu: MMU,
    pub cartrige: Option<Box<dyn Cartridge>>,
    pub current_rom_bank: usize,
    pub current_ram_bank: usize,
    pub rom_banking: bool,
    pub enable_ram: bool,
    pub timer_counter: u32,
    pub divider_counter: u16,
    pub interrupts_enabled: bool,
    pub pending_interrupt: Option<bool>,
    pub scanline_counter: u16,
    pub halted: bool,
    pub video_buffer: Vec<u32>,
    pub joypad_state: u8,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: CPU::new(),
            mmu: MMU::new(),
            cartrige: None,
            current_rom_bank: 1,
            current_ram_bank: 0,
            enable_ram: false,
            rom_banking: false,
            timer_counter: 0,
            divider_counter: 0,
            interrupts_enabled: false,
            pending_interrupt: None,
            scanline_counter: 0,
            halted: false,
            video_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            joypad_state: 0xcf,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        self.cartrige = load_rom(path).ok();
    }

    pub fn update(&mut self) {
        let cycles = self.execute_next_opcode();
        self.update_timers(cycles);
        self.update_graphics(cycles);
        self.do_interrupts();
    }

    pub fn execute_next_opcode(&mut self) -> u16 {
        let cycles = if self.halted {
            4
        } else {
            let opcode = self.read_memory(self.cpu.pc);
            self.cpu.pc += 1;
            let result = self.execute(opcode);
            result
        };

        if self.pending_interrupt == Some(true) && self.read_memory(self.cpu.pc - 1) != 0xFB {
            self.interrupts_enabled = true;
            self.pending_interrupt = None;
        } else if self.pending_interrupt == Some(false) && self.read_memory(self.cpu.pc - 1) != 0xF3
        {
            self.interrupts_enabled = false;
            self.pending_interrupt = None;
        }

        cycles
    }

    pub fn push_stack(&mut self, data: u16) {
        self.write_memory(self.cpu.sp.wrapping_sub(1), data.hi());
        self.write_memory(self.cpu.sp.wrapping_sub(2), data.lo());
        self.cpu.sp = self.cpu.sp.wrapping_sub(2);
    }

    pub fn pop_stack(&mut self) -> u16 {
        let lo = self.read_memory(self.cpu.sp);
        let hi = self.read_memory(self.cpu.sp.wrapping_add(1));
        self.cpu.sp = self.cpu.sp.wrapping_add(2);
        u16::from_bytes(hi, lo)
    }

    pub fn read_memory(&self, address: u16) -> u8 {
        if (address < 0x8000) || (address >= 0xA000 && address < 0xC000) {
            return self.cartrige.as_ref().unwrap().read(address as usize);
        } else if address >= 0xE000 && address < 0xFE00 {
            // echo ram
            return self.read_memory(address - 0x2000);
        } else if address == DIV {
            // divider register
            return self.divider_counter.hi();
        }
        if address == 0xFF00 {
            return self.get_joypad_state();
        }
        return self.mmu.read_byte(address);
    }

    pub fn write_memory(&mut self, address: u16, data: u8) {
        if (address < 0x8000) || (address >= 0xA000 && address < 0xC000) {
            return self
                .cartrige
                .as_mut()
                .unwrap()
                .write(address as usize, data);
        } else if address >= 0xFEA0 && address < 0xFEFF {
            return; // prohibited
        } else if address >= 0xE000 && address < 0xFE00 {
            // echo ram
            self.write_memory(address - 0x2000, data);
            return;
        } else if address == TMC {
            let before = self.read_memory(TMC);
            self.mmu.write_byte(TMC, data);
            if data != before {
                self.timer_counter = 0;
            }
            return;
        } else if address == DIV {
            self.divider_counter = 0;
            self.timer_counter = 0;
            return;
        } else if address == SCANLINE {
            self.mmu.write_byte(SCANLINE, 0);
            return;
        } else if address == DMA {
            self.dma_transfer(data);
            return;
        } else if address == 0xFF01 {
            // print serial output
            print!("{}", data as char);
        }
        self.mmu.write_byte(address, data);
    }
}

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
        // result to option
        self.cartrige = load_rom(path).ok();
    }

    pub fn update(&mut self) {
        let cycles = self.execute_next_opcode();
        self.update_timers(cycles);
        self.update_graphics(cycles);
        self.do_interrupts();
    }

    pub fn log(&self, log: &str) {
        // write the current cpu state to the log file
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("log.txt")
            .unwrap();
        // E.g.: A:00 F:11 B:22 C:33 D:44 E:55 H:66 L:77 SP:8888 PC:9999 PCMEM:AA,BB,CC,DD
        // let log = format!(
        //     "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
        //     self.cpu.a,
        //     self.cpu.f,
        //     self.cpu.b,
        //     self.cpu.c,
        //     self.cpu.d,
        //     self.cpu.e,
        //     self.cpu.h,
        //     self.cpu.l,
        //     self.cpu.sp,
        //     self.cpu.pc,
        //     self.read_memory(self.cpu.pc),
        //     self.read_memory(self.cpu.pc + 1),
        //     self.read_memory(self.cpu.pc + 2),
        //     self.read_memory(self.cpu.pc + 3),
        // );
        file.write_all(log.as_bytes()).unwrap();
    }

    pub fn execute_next_opcode(&mut self) -> u16 {
        let cycles = if self.halted {
            4
        } else {
            let opcode = self.read_memory(self.cpu.pc);
            // self.log(&format!("{:04X} {:02X}\n", self.cpu.pc, opcode));
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

    fn update_timers(&mut self, cycles: u16) {
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

    fn update_graphics(&mut self, cycles: u16) {
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

    fn draw_scanline(&mut self) {
        let control = self.read_memory(LCD_CONTROL);
        if control.test_bit(0) {
            self.render_tiles()
        }
        if control.test_bit(1) {
            self.render_sprites()
        }
    }

    fn render_tiles(&mut self) {
        let control = self.read_memory(LCD_CONTROL);

        let scroll_y = self.read_memory(0xFF42);
        let scroll_x = self.read_memory(0xFF43);
        let window_y = self.read_memory(0xFF4A);
        let window_x = self.read_memory(0xFF4B).wrapping_sub(7);

        let using_window =
            self.read_memory(0xFF40).test_bit(5) && window_y <= self.read_memory(0xFF44);
        let unsigned = control.test_bit(4);
        let tile_data: u16 = if unsigned { 0x8000 } else { 0x8800 };

        let background_mem = if using_window {
            if control.test_bit(6) {
                0x9C00
            } else {
                0x9800
            }
        } else {
            if control.test_bit(3) {
                0x9C00
            } else {
                0x9800
            }
        };

        let y_pos = if using_window {
            self.read_memory(0xFF44).wrapping_sub(window_y)
        } else {
            scroll_y.wrapping_add(self.read_memory(0xFF44))
        };

        let tile_row = ((y_pos / 8) as u16) * 32;

        for pixel in 0_u8..160_u8 {
            let mut x_pos = pixel.wrapping_add(scroll_x);

            if using_window && pixel >= window_x {
                x_pos = pixel - window_x;
            }

            let tile_col = (x_pos / 8) as u16;
            let tile_address = background_mem + tile_row + tile_col;
            let tile_location = if unsigned {
                let tile_num = self.read_memory(tile_address) as u16;
                tile_data + tile_num * 16
            } else {
                let tile_num = self.read_memory(tile_address) as i8 as i16;
                tile_data.wrapping_add_signed((tile_num + 128) * 16)
            };

            let line = ((y_pos % 8) * 2) as u16;
            let data1 = self.read_memory(tile_location + line);
            let data2 = self.read_memory(tile_location + line + 1);

            let color_bit = ((x_pos % 8) as i16 - 7) * -1;
            let color_num = ((data2.test_bit(color_bit as u8) as u8) << 1)
                | (data1.test_bit(color_bit as u8) as u8);
            let color = self.get_color(color_num, 0xFF47);

            let final_y = self.read_memory(0xFF44);

            if final_y > 143 || pixel > 159 {
                continue;
            }

            self.video_buffer[final_y as usize * SCREEN_WIDTH + pixel as usize] = color;
        }
    }

    fn render_sprites(&mut self) {
        let control = self.read_memory(LCD_CONTROL);
        let using_8x16 = control.test_bit(2);

        for sprite in 0..40 {
            let index: u16 = sprite * 4;

            let y_pos = self.read_memory(0xFE00 + index).wrapping_sub(16);
            let x_pos = self.read_memory(0xFE00 + index + 1).wrapping_sub(8);

            let tile_location = self.read_memory(0xFE00 + index + 2);
            let attributes = self.read_memory(0xFE00 + index + 3);

            let y_flip = attributes.test_bit(6);
            let x_flip = attributes.test_bit(5);

            let scanline = self.read_memory(0xFF44);
            let y_size = if using_8x16 { 16 } else { 8 };

            if scanline >= y_pos && scanline < y_pos + y_size {
                let mut line = (scanline - y_pos) as i32;
                if y_flip {
                    line -= y_size as i32;
                    line *= -1;
                }

                line *= 2;

                let address = 0x8000 + (tile_location as u16 * 16) + line as u16;

                let data1 = self.read_memory(address);
                let data2 = self.read_memory(address + 1);

                for tile_pixel in (0_u8..8_u8).rev() {
                    let mut color_bit = tile_pixel;
                    if x_flip {
                        color_bit = ((tile_pixel as i8 - 7) * -1) as u8;
                    }
                    let color_num = ((data2.test_bit(color_bit) as u8) << 1)
                        | (data1.test_bit(color_bit) as u8);
                    let color_address = if attributes.test_bit(4) {
                        0xFF49
                    } else {
                        0xFF48
                    };
                    let color = self.get_color(color_num, color_address);

                    if color == COLOR_WHITE {
                        continue;
                    }

                    let final_y = self.read_memory(0xFF44);
                    let pixel = x_pos.wrapping_add(7 - tile_pixel);

                    if final_y > 143 || pixel > 159 {
                        continue;
                    }

                    self.video_buffer[final_y as usize * SCREEN_WIDTH + pixel as usize] = color;
                }
            }
        }
    }

    fn get_color(&self, color_num: u8, address: u16) -> u32 {
        let palette = self.read_memory(address);
        let (hi, lo) = match color_num {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 4),
            3 => (7, 6),
            _ => panic!("Invalid color number"),
        };

        let color = ((palette.test_bit(hi) as u8) << 1) | palette.test_bit(lo) as u8;

        match color {
            0 => COLOR_WHITE,
            1 => COLOR_LIGHT_GRAY,
            2 => COLOR_DARK_GRAY,
            3 => COLOR_BLACK,
            _ => panic!("Invalid color"),
        }
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
        // read memory
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
            let frequency = self.get_clock_frequency();
            self.mmu.write_byte(TMC, data);
            let new_frequency = self.get_clock_frequency();
            if frequency != new_frequency {
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

    fn get_joypad_state(&self) -> u8 {
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

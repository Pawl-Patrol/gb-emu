use crate::{canvas::*, constants::*, emulator::Emulator, traits::*};

impl Emulator {
    pub fn update_graphics(&mut self, cycles: u16) {
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
        let mode;
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

        if request_interrupt && (mode != current_mode) {
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
                let tile_num = self.read_memory(tile_address);
                tile_data + ((tile_num as i8 as i16 + 128) * 16) as u16
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

            if scanline < y_pos || scanline >= y_pos + y_size {
                continue;
            }

            let mut line = scanline - y_pos;
            if y_flip {
                line = y_size - line;
            }

            line *= 2;

            let address = 0x8000 + (tile_location as u16 * 16) + line as u16;

            let data1 = self.read_memory(address);
            let data2 = self.read_memory(address + 1);

            for tile_pixel in (0_u8..8_u8).rev() {
                let mut color_bit = tile_pixel;
                if x_flip {
                    color_bit = 7 - color_bit;
                }
                let color_num =
                    ((data2.test_bit(color_bit) as u8) << 1) | (data1.test_bit(color_bit) as u8);
                let color_address = if attributes.test_bit(4) {
                    0xFF49
                } else {
                    0xFF48
                };
                let mut color = self.get_color(color_num, color_address);

                if color == COLOR_WHITE {
                    continue;
                }

                let pixel = x_pos.wrapping_add(7 - tile_pixel);

                if scanline > 143 || pixel > 159 {
                    continue;
                }

                self.video_buffer[scanline as usize * SCREEN_WIDTH + pixel as usize] = color;
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

    pub fn dma_transfer(&mut self, data: u8) {
        let address = (data as u16) << 8;
        for i in 0..0xA0 {
            let value = self.read_memory(address + i);
            self.write_memory(0xFE00 + i, value);
        }
    }
}

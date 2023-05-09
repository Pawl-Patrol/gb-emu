use crate::{canvas::*, constants::*, traits::*};

pub struct GPU {
    pub vram: [u8; 0x2000], // video ram
    pub oam: [u8; 0xA0],    // object attribute memory
    pub video_buffer: Vec<u32>,
    pub scanline_counter: u16,
    pub needs_interrupt: Option<u8>,
    pub lcd_control: u8,
    pub lcd_status: u8,
    pub scroll_y: u8,
    pub scroll_x: u8,
    pub window_y: u8,
    pub window_x: u8,
    pub ly: u8,
    pub ly_compare: u8,
    pub bg_palette: u8,
    pub obj_palette_1: u8,
    pub obj_palette_2: u8,
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            video_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            scanline_counter: 0,
            needs_interrupt: None,
            lcd_control: 0x91,
            lcd_status: 0x85,
            scroll_y: 0x00,
            scroll_x: 0x00,
            ly: 0x00,
            ly_compare: 0x00,
            bg_palette: 0xFC,
            obj_palette_1: 0xFF,
            obj_palette_2: 0xFF,
            window_y: 0x00,
            window_x: 0x00,
        }
    }
    pub fn update_graphics(&mut self, cycles: u16) {
        self.set_lcd_status();

        if !self.lcd_enabled() {
            return;
        }

        if let Some(new_scanline_counter) = self.scanline_counter.checked_sub(cycles) {
            self.scanline_counter = new_scanline_counter;
        } else {
            self.scanline_counter = SCANLINE_CYCLES;
            self.ly += 1;

            if self.ly == 144 {
                self.needs_interrupt = Some(0);
            } else if self.ly > 153 {
                self.ly = 0;
            } else if self.ly < 144 {
                self.draw_scanline();
            }
        }
    }

    fn set_lcd_status(&mut self) {
        if !self.lcd_enabled() {
            self.scanline_counter = SCANLINE_CYCLES;
            self.ly = 0;
            self.lcd_status &= 0b1111_1100;
            self.lcd_status.set_bit(0);
            return;
        }

        let current_mode = self.lcd_status & 0b0000_0011;
        let mode;
        let mut request_interrupt = false;

        if self.ly >= 144 {
            mode = 1;
            self.lcd_status.set_bit(0);
            self.lcd_status.reset_bit(1);
            request_interrupt = self.lcd_status.test_bit(4);
        } else {
            let mode2_bounds = SCANLINE_CYCLES - 80;
            let mode3_bounds = mode2_bounds - 172;

            if self.scanline_counter >= mode2_bounds {
                mode = 2;
                self.lcd_status.set_bit(1);
                self.lcd_status.reset_bit(0);
                request_interrupt = self.lcd_status.test_bit(5);
            } else if self.scanline_counter >= mode3_bounds {
                mode = 3;
                self.lcd_status.set_bit(1);
                self.lcd_status.set_bit(0);
            } else {
                mode = 0;
                self.lcd_status.reset_bit(1);
                self.lcd_status.reset_bit(0);
                request_interrupt = self.lcd_status.test_bit(3);
            }
        }

        if request_interrupt && (mode != current_mode) {
            self.needs_interrupt = Some(1);
        }

        if self.ly == self.ly_compare {
            self.lcd_status.set_bit(2);
            if self.lcd_status.test_bit(6) {
                self.needs_interrupt = Some(1);
            }
        } else {
            self.lcd_status.reset_bit(2);
        }
    }

    fn lcd_enabled(&self) -> bool {
        self.lcd_control.test_bit(7)
    }

    fn draw_scanline(&mut self) {
        if self.lcd_control.test_bit(0) {
            self.render_tiles()
        }
        if self.lcd_control.test_bit(1) {
            self.render_sprites()
        }
    }

    fn render_tiles(&mut self) {
        let using_window = self.lcd_control.test_bit(5) && (self.window_y <= self.ly);
        let unsigned = self.lcd_control.test_bit(4);
        let tile_data: u16 = if unsigned { 0x8000 } else { 0x8800 };

        let background_mem = if using_window {
            if self.lcd_control.test_bit(6) {
                0x9C00
            } else {
                0x9800
            }
        } else {
            if self.lcd_control.test_bit(3) {
                0x9C00
            } else {
                0x9800
            }
        };

        let y_pos = if using_window {
            self.ly.wrapping_sub(self.window_y)
        } else {
            self.scroll_y.wrapping_add(self.ly)
        };

        let tile_row = ((y_pos / 8) as u16) * 32;

        for pixel in 0_u8..160_u8 {
            let mut x_pos = pixel.wrapping_add(self.scroll_x);

            if using_window && (pixel >= self.window_x) {
                x_pos = pixel - self.window_x;
            }

            let tile_address = background_mem + tile_row + (x_pos / 8) as u16;
            let tile_num = self.read(tile_address as usize);
            let tile_location = if unsigned {
                tile_data + (tile_num as u16 * 16)
            } else {
                tile_data + ((tile_num as i8 as i16 + 128) as u16 * 16)
            };

            let line = ((y_pos % 8) * 2) as u16;
            let data1 = self.read((tile_location + line) as usize);
            let data2 = self.read((tile_location + line + 1) as usize);

            let color_bit = 7 - (x_pos % 8);
            let color_num =
                ((data2.test_bit(color_bit) as u8) << 1) | (data1.test_bit(color_bit) as u8);
            let color = self.get_color(color_num, 0xFF47);

            if self.ly > 143 || pixel > 159 {
                continue;
            }

            self.video_buffer[self.ly as usize * SCREEN_WIDTH + pixel as usize] = color;
        }
    }

    fn render_sprites(&mut self) {
        let using_8x16 = self.lcd_control.test_bit(2);

        for sprite in 0..40 {
            let index = 0xFE00 + sprite as usize * 4;

            let y_pos = self.read(index).wrapping_sub(16);
            let x_pos = self.read(index + 1).wrapping_sub(8);

            let tile_location = self.read(index + 2);
            let attributes = self.read(index + 3);

            let y_flip = attributes.test_bit(6);
            let x_flip = attributes.test_bit(5);

            let y_size = if using_8x16 { 16 } else { 8 };

            if (self.ly < y_pos) || (self.ly >= y_pos + y_size) {
                continue;
            }

            let mut line = self.ly - y_pos;

            if y_flip {
                line = y_size - line;
            }

            line *= 2;

            let address = 0x8000 + (tile_location as u16 * 16) + line as u16;

            let data1 = self.read(address as usize);
            let data2 = self.read(address as usize + 1);

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

                let color = self.get_color(color_num, color_address);

                if color == COLOR_WHITE {
                    continue;
                }

                let pixel = x_pos.wrapping_add(7).wrapping_sub(tile_pixel);

                if self.ly > 143 || pixel > 159 {
                    continue;
                }

                let video_buffer_index = self.ly as usize * SCREEN_WIDTH + pixel as usize;

                if attributes.test_bit(7) && self.video_buffer[video_buffer_index] != COLOR_WHITE {
                    continue;
                }

                self.video_buffer[video_buffer_index] = color;
            }
        }
    }

    fn get_color(&self, color_num: u8, address: u16) -> u32 {
        let palette = self.read(address as usize);
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
}

impl Cartridge for GPU {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[address - 0x8000],
            0xFE00..=0xFE9F => self.oam[address - 0xFE00],
            0xFF40 => self.lcd_control,
            0xFF41 => self.lcd_status,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.ly_compare,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette_1,
            0xFF49 => self.obj_palette_2,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Invalid gpu address: 0x{:X}", address),
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[address - 0x8000] = value,
            0xFE00..=0xFE9F => self.oam[address - 0xFE00] = value,
            0xFF40 => self.lcd_control = value,
            0xFF41 => self.lcd_status = value,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.ly = 0,
            0xFF45 => self.ly_compare = value,
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette_1 = value,
            0xFF49 => self.obj_palette_2 = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => panic!("Invalid gpu address: 0x{:X}", address),
        }
    }
}

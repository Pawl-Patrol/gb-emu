use crate::traits::*;

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        ($r << 16) | ($g << 8) | $b
    };
}

pub const COLOR_WHITE: u32 = rgb!(0xFF, 0xFF, 0xFF);
pub const COLOR_LIGHT_GRAY: u32 = rgb!(0xCC, 0xCC, 0xCC);
pub const COLOR_DARK_GRAY: u32 = rgb!(0x77, 0x77, 0x77);
pub const COLOR_BLACK: u32 = rgb!(0x00, 0x00, 0x00);

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const MODE_HBLANK: u8 = 0b00;
pub const MODE_VBLANK: u8 = 0b01;
pub const MODE_OAM: u8 = 0b10;
pub const MODE_VRAM: u8 = 0b11;

type Tile = [[u8; 8]; 8];

#[derive(Default, Copy, Clone)]
pub struct Sprite {
    y: u8,
    x: u8,
    tile_index: u8,
    bg_priority: bool,
    y_flip: bool,
    x_flip: bool,
    palette: bool,
}

pub struct GPU {
    pub vram: [u8; 0x2000],    // 8KB of video ram
    pub oam: [u8; 160],        // 160 bytes of sprite attribute memory
    pub tiles: [Tile; 384],    // 384 tiles, each tile is 8x8 pixels
    pub sprites: [Sprite; 40], // 40 sprites
    pub video_buffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub cycles: u16,
    pub scanline_counter: u16,
    pub lcd_control: u8,
    pub lcd_status: u8,
    pub scroll_y: u8,
    pub scroll_x: u8,
    pub window_y: u8,
    pub window_x: u8,
    pub ly: u8,
    pub ly_compare: u8,
    pub bg_palette: u8,
    pub obj_palette_0: u8,
    pub obj_palette_1: u8,
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
            0xFF48 => self.obj_palette_0,
            0xFF49 => self.obj_palette_1,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => 0x00,
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                let address = address - 0x8000;
                self.vram[address] = value;
                if address < 0x1800 {
                    self.update_tile(address);
                }
            }
            0xFE00..=0xFE9F => {
                let address = address - 0xFE00;
                self.oam[address] = value;
                if address < 0xA0 {
                    self.update_sprite(address);
                }
            }
            0xFF40 => self.lcd_control = value,
            0xFF41 => self.lcd_status = value,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.ly = 0,
            0xFF45 => self.ly_compare = value,
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette_0 = value,
            0xFF49 => self.obj_palette_1 = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => (), // println!("Invalid gpu address: 0x{:04X} = {:02X}", address, value),
        }
    }
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            vram: [0; 0x2000],
            oam: [0; 160],
            sprites: [Sprite::default(); 40],
            tiles: [[[0; 8]; 8]; 384],
            video_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            cycles: 0,
            scanline_counter: 0,
            lcd_control: 0x91,
            lcd_status: 0x85,
            scroll_y: 0x00,
            scroll_x: 0x00,
            ly: 0x00,
            ly_compare: 0x00,
            bg_palette: 0xFC,
            obj_palette_0: 0xFF,
            obj_palette_1: 0xFF,
            window_y: 0x00,
            window_x: 0x00,
        }
    }

    pub fn update_graphics(&mut self, cycles: u16) -> u8 {
        let mut needs_interrupt = 0;

        if !self.lcd_enabled() {
            return needs_interrupt;
        }

        self.cycles += cycles;

        match self.mode() {
            MODE_HBLANK => {
                if self.cycles > 200 {
                    self.cycles -= 200;
                    self.ly += 1;
                    if self.ly >= 144 {
                        self.set_mode(MODE_VBLANK);
                        needs_interrupt |= 1; // VBLANK interrupt
                        if self.lcd_status.test_bit(4) {
                            needs_interrupt |= 2; // LCD STAT interrupt
                        }
                    } else {
                        self.set_mode(MODE_OAM);
                        if self.lcd_status.test_bit(5) {
                            needs_interrupt |= 2; // LCD STAT interrupt
                        }
                    }
                    needs_interrupt |= self.compare_ly_lyc();
                }
            }
            MODE_VBLANK => {
                if self.cycles >= 456 {
                    self.cycles %= 456;
                    self.ly += 1;
                    if self.ly == 154 {
                        self.ly = 0;
                        self.set_mode(MODE_OAM);
                        if self.lcd_status.test_bit(5) {
                            needs_interrupt |= 2; // LCD STAT interrupt
                        }
                    }
                    needs_interrupt |= self.compare_ly_lyc();
                }
            }
            MODE_OAM => {
                if self.cycles >= 80 {
                    self.cycles %= 80;
                    self.set_mode(MODE_VRAM);
                }
            }
            MODE_VRAM => {
                if self.cycles >= 172 {
                    self.cycles %= 172;
                    self.set_mode(MODE_HBLANK);
                    if self.lcd_status.test_bit(3) {
                        needs_interrupt |= 2; // LCD STAT interrupt
                    }
                    self.render_scanline();
                }
            }
            _ => unreachable!(),
        }

        needs_interrupt
    }

    fn get_color(palette: u8, color: u8) -> u32 {
        match (palette >> (color * 2)) & 0b11 {
            0b00 => COLOR_WHITE,
            0b01 => COLOR_LIGHT_GRAY,
            0b10 => COLOR_DARK_GRAY,
            0b11 => COLOR_BLACK,
            _ => unreachable!(),
        }
    }

    fn compare_ly_lyc(&mut self) -> u8 {
        let result = self.ly == self.ly_compare;
        self.lcd_status.toggle_bit(2, result); // set ly=lyc flag

        // if ly=lyc interrupt is enabled
        if result && self.lcd_status.test_bit(6) {
            2 // LCD STAT interrupt
        } else {
            0
        }
    }

    fn mode(&self) -> u8 {
        self.lcd_status & 0b11
    }

    fn set_mode(&mut self, mode: u8) {
        self.lcd_status = (self.lcd_status & 0b1111_1100) | mode;
    }

    fn lcd_enabled(&self) -> bool {
        self.lcd_control.test_bit(7)
    }

    fn update_tile(&mut self, address: usize) {
        let normalized = address & 0xFFFE; // round even
        let data1 = self.vram[normalized];
        let data2 = self.vram[normalized + 1];
        let tile = address / 16;
        let y = (address % 16) / 2;
        for x in 0..8 {
            let bit1 = (data1 >> (7 - x)) & 0b1;
            let bit2 = (data2 >> (7 - x)) & 0b1;
            let color = (bit2 << 1) | bit1;
            self.tiles[tile][y][x] = color;
        }
    }

    fn update_sprite(&mut self, address: usize) {
        let address = address & 0xFFFC; // round to multiple of 4
        let sprite = address / 4;
        self.sprites[sprite].y = self.oam[address].wrapping_sub(16);
        self.sprites[sprite].x = self.oam[address + 1].wrapping_sub(8);
        self.sprites[sprite].tile_index = self.oam[address + 2];
        let flags = self.oam[address + 3];
        self.sprites[sprite].bg_priority = flags.test_bit(7);
        self.sprites[sprite].y_flip = flags.test_bit(6);
        self.sprites[sprite].x_flip = flags.test_bit(5);
        self.sprites[sprite].palette = flags.test_bit(4);
    }

    fn render_scanline(&mut self) {
        if self.lcd_control.test_bit(0) {
            self.render_tiles();
        }
        if self.lcd_control.test_bit(1) {
            self.render_sprites();
        }
    }

    fn render_tiles(&mut self) {
        let using_window = self.lcd_control.test_bit(5) && self.window_y <= self.ly;
        let unsigned = self.lcd_control.test_bit(4);

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

        let y = if using_window {
            self.ly.wrapping_sub(self.window_y)
        } else {
            self.ly.wrapping_add(self.scroll_y)
        };

        let tile_row = (y / 8) as u16 * 32;

        let corrected_window_x = self.window_x.wrapping_sub(7);

        for pixel in 0_u8..160_u8 {
            let x = if using_window && pixel >= corrected_window_x {
                pixel.wrapping_sub(corrected_window_x)
            } else {
                pixel.wrapping_add(self.scroll_x)
            };
            let tile_col = (x / 8) as u16;
            let tile_address = background_mem + tile_row + tile_col;
            let tile_offset = self.read(tile_address as usize);
            let tile_index = if unsigned {
                tile_offset as usize
            } else {
                tile_offset.wrapping_add(128) as usize + 128
            };

            let tile_color_index = self.tiles[tile_index][y as usize % 8][x as usize % 8];
            let color = GPU::get_color(self.bg_palette, tile_color_index);

            self.video_buffer[self.ly as usize * 160 + pixel as usize] = color;
        }
    }

    fn render_sprites(&mut self) {
        let using_8x16 = self.lcd_control.test_bit(2);

        for sprite in &self.sprites {
            let size_y = if using_8x16 { 16 } else { 8 };
            if self.ly < sprite.y || self.ly >= sprite.y + size_y {
                continue;
            }
            let mut y = (self.ly - sprite.y) % 8; // modulo 8 because of 8x16 sprites
            if sprite.y_flip {
                y = 7 - y;
            }

            let tile = self.tiles[sprite.tile_index as usize];

            for pixel in 0..8 {
                let mut x = pixel;
                if sprite.x_flip {
                    x = 7 - x;
                }
                let color_index = tile[y as usize][x as usize];
                if color_index == 0 {
                    continue;
                }
                let palette = if sprite.palette {
                    self.obj_palette_1
                } else {
                    self.obj_palette_0
                };
                let color = GPU::get_color(palette, color_index);
                let index = self.ly as usize * 160 + sprite.x.wrapping_add(pixel) as usize;
                if !sprite.bg_priority || self.video_buffer[index] == COLOR_WHITE {
                    self.video_buffer[index] = color;
                }
            }
        }
    }
}

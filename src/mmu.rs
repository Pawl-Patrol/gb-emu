use crate::{
    cartridge::{load_rom, Cartridge},
    constants::{DIV, DMA, SCANLINE, TMC},
    context::Context,
    traits::Register,
};

pub struct MMU {
    pub cartrige: Option<Box<dyn Cartridge>>,
    pub is_in_bios: bool,
    pub bios: [u8; 0x100],  // biosj
    pub wram: [u8; 0x2000], // work ram
    pub oam: [u8; 0xA0],    // object attribute memory
    pub io: [u8; 0x80],     // io ports
    pub hram: [u8; 0x7F],   // high ram
    pub vram: [u8; 0x2000], // video ram
    pub interrupt_enable: u8,
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            is_in_bios: false,
            cartrige: None,
            bios: [0; 0x100],
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            interrupt_enable: 0,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        self.cartrige = load_rom(path).ok();
    }

    pub fn read_byte(&self, ctx: &Context, address: u16) -> u8 {
        match address {
            DIV => ctx.rtc.divider_counter.hi(),
            0xFF00 => ctx.joypad.get_joypad_state(ctx), // TODO: move to constant
            0x0000..=0x00FF if self.is_in_bios => self.bios[address as usize],
            0x0000..=0x7FFF | 0xA000..=0xBFFF => {
                self.cartrige.as_ref().unwrap().read(address as usize)
            }
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0x00, // prohibited
            0xFF00..=0xFF7F => self.io[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupt_enable,
        }
    }

    pub fn write_byte(&mut self, ctx: &mut Context, address: u16, value: u8) {
        match address {
            TMC => {
                let before = self.read_byte(ctx, TMC);
                self.io[(address - 0xFF00) as usize] = value;
                if value != before {
                    ctx.rtc.timer_counter = 0;
                }
            }
            DIV => {
                ctx.rtc.divider_counter = 0;
                ctx.rtc.timer_counter = 0;
            }
            SCANLINE => self.io[(SCANLINE - 0xFF00) as usize] = 0,
            DMA => ctx.gpu.dma_transfer(ctx, value),
            0xFF01 => print!("{}", value as char), // print serial output
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self
                .cartrige
                .as_mut()
                .unwrap()
                .write(address as usize, value),
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFEA0..=0xFEFF => (), // prohibited
            0xFF00..=0xFF7F => self.io[(address - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupt_enable = value,
        }
    }
}

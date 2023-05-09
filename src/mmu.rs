use crate::{
    cartridge::load_rom, constants::DMA, gpu::GPU, joypad::JoyPad, rtc::RTC, traits::Cartridge,
};

pub struct MMU {
    pub cartrige: Option<Box<dyn Cartridge>>,
    pub gpu: GPU,
    pub rtc: RTC,
    pub joypad: JoyPad,

    pub is_in_bios: bool,
    pub bios: [u8; 0x100],  // biosj
    pub wram: [u8; 0x2000], // work ram
    pub hram: [u8; 0x7F],   // high ram
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,
    pub io_backup: [u8; 0x80],
    pub dma: u8,
}

impl MMU {
    pub fn new() -> MMU {
        let mut io_backup = [0; 0x80];

        io_backup[0x05] = 0x00;
        io_backup[0x06] = 0x00;
        io_backup[0x07] = 0x00;
        io_backup[0x10] = 0x80;
        io_backup[0x11] = 0xBF;
        io_backup[0x12] = 0xF3;
        io_backup[0x14] = 0xBF;
        io_backup[0x16] = 0x3F;
        io_backup[0x17] = 0x00;
        io_backup[0x19] = 0xBF;
        io_backup[0x1A] = 0x7F;
        io_backup[0x1B] = 0xFF;
        io_backup[0x1C] = 0x9F;
        io_backup[0x1E] = 0xBF;
        io_backup[0x20] = 0xFF;
        io_backup[0x21] = 0x00;
        io_backup[0x22] = 0x00;
        io_backup[0x23] = 0xBF;
        io_backup[0x24] = 0x77;
        io_backup[0x25] = 0xF3;
        io_backup[0x26] = 0xF1;
        io_backup[0x40] = 0x91;
        io_backup[0x42] = 0x00;
        io_backup[0x43] = 0x00;
        io_backup[0x45] = 0x00;
        io_backup[0x47] = 0xFC;
        io_backup[0x48] = 0xFF;
        io_backup[0x49] = 0xFF;
        io_backup[0x4A] = 0x00;
        io_backup[0x4B] = 0x00;

        MMU {
            cartrige: None,
            gpu: GPU::new(),
            rtc: RTC::new(),
            joypad: JoyPad::new(),
            is_in_bios: false,
            bios: [0; 0x100],
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            interrupt_enable: 0x00,
            interrupt_flag: 0xE1,
            io_backup,
            dma: 0xFF,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        self.cartrige = load_rom(path).ok();
    }

    pub fn dma_transfer(&mut self, data: u8) {
        self.dma = data;
        let address = (data as u16) << 8;
        for i in 0..0xA0 {
            let value = self.read(address + i);
            self.gpu.oam[i as usize] = value;
        }
    }

    // TODO: replace u16 with usize
    pub fn read(&self, address: u16) -> u8 {
        match address {
            // bios
            0x0000..=0x00FF if self.is_in_bios => self.bios[address as usize],
            // rom
            0x0000..=0x7FFF | 0xA000..=0xBFFF => {
                self.cartrige.as_ref().unwrap().read(address as usize)
            }
            // DMA
            0xFF46 => self.dma,
            // gpu
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4F | 0xFF68..=0xFF6B => {
                self.gpu.read(address as usize)
            }
            // rtc
            0xFF04..=0xFF07 => self.rtc.read(address as usize),
            // IF
            0xFF0F => self.interrupt_flag,
            // work ram
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            // prohibited
            0xFEA0..=0xFEFF => 0x00,
            // high ram
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            // joypad
            0xFF00 => self.joypad.read(address as usize),
            // IE
            0xFFFF => self.interrupt_enable,
            // backup
            0xFF00..=0xFF7F => {
                println!("read from io: {:04X}", address);
                self.io_backup[(address - 0xFF00) as usize]
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // rom
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self
                .cartrige
                .as_mut()
                .unwrap()
                .write(address as usize, value),
            // DMA
            0xFF46 => self.dma_transfer(value),
            // gpu
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4F | 0xFF68..=0xFF6B => {
                self.gpu.write(address as usize, value)
            }
            // rtc
            0xFF04..=0xFF07 => self.rtc.write(address as usize, value),
            // work ram
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            // joypad
            0xFF00 => self.joypad.write(address as usize, value),
            // prohibited
            0xFEA0..=0xFEFF => (),
            // high ram
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            // IF
            0xFF0F => self.interrupt_flag = value,
            // serial
            0xFF01 => print!("{}", value as char), // print serial output
            // interrupt flags
            0xFFFF => self.interrupt_enable = value,
            // backup
            0xFF00..=0xFF7F => {
                println!("write to io: {:X} = {:X}", address, value);
                self.io_backup[(address - 0xFF00) as usize] = value;
            }
        }
    }
}

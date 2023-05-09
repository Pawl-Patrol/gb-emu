use crate::constants::*;
use std::fs::read;

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub bank_mode: u8,
}

fn get_bank_mode(data: u8) -> u8 {
    match data {
        0x00 | 0x08 | 0x09 => NO_MBC,
        0x01 | 0x02 | 0x03 => MBC1,
        0x05 | 0x06 => MBC2,
        0x0D | 0x0F | 0x10 | 0x11 | 0x12 | 0x13 => MBC3,
        0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => MBC5,
        _ => panic!("Unsupported cartridge type"),
    }
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            rom: Vec::new(),
            ram: Vec::new(),
            bank_mode: 0,
        }
    }

    pub fn load(&mut self, path: &str) {
        self.rom = read(path).unwrap();
        self.ram = vec![0; 0x8000];
        println!("Detected cartridge type: ${:02X}", self.rom[0x147]);
        self.bank_mode = get_bank_mode(self.rom[0x147]);
    }
}

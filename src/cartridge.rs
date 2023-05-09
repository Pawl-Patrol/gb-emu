use crate::constants::*;
use std::fs::read;

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub bank_mode: u8,
}

fn get_bank_mode(data: u8) -> u8 {
    match data {
        1 | 2 | 3 => MBC1,
        5 | 6 => MBC2,
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
        let banks = self.rom[0x148] as usize;
        self.ram = vec![0; RAM_BANK_SIZE * banks];
        self.bank_mode = get_bank_mode(self.rom[0x147]);
    }
}

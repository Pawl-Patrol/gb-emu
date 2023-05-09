use std::io::{Read, Result, Seek};

const REGISTER_CARTRIDGE_TYPE: usize = 0x0147;
const REGISTER_ROM_SIZE: usize = 0x0148;
const REGISTER_RAM_SIZE: usize = 0x0149;
const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

pub fn load_rom(path: &str) -> Result<Box<dyn Cartridge>> {
    let mut file = std::fs::File::open(path)?;
    file.seek(std::io::SeekFrom::Start(REGISTER_CARTRIDGE_TYPE as u64))?;
    let mut buffer = [0_u8; 1];
    file.read_exact(&mut buffer)?;
    println!("Cartridge type: {:#04X}", buffer[0]);
    match buffer[0] {
        0x00 | 0x08 | 0x09 => Ok(Box::new(NoMBC::load(path))),
        0x01 | 0x02 | 0x03 => Ok(Box::new(MBC1::load(path))),
        _ => panic!("Unsupported cartridge type!"),
    }
}

fn get_rom_size(value: u8) -> usize {
    match value {
        0x00 => ROM_BANK_SIZE * 2,
        0x01 => ROM_BANK_SIZE * 4,
        0x02 => ROM_BANK_SIZE * 8,
        0x03 => ROM_BANK_SIZE * 16,
        0x04 => ROM_BANK_SIZE * 32,
        0x05 => ROM_BANK_SIZE * 64,
        0x06 => ROM_BANK_SIZE * 128,
        0x07 => ROM_BANK_SIZE * 256,
        0x08 => ROM_BANK_SIZE * 512,
        _ => panic!("Unused rom size!"),
    }
}

fn get_ram_size(value: u8) -> usize {
    match value {
        0x00 => 0,
        0x02 => RAM_BANK_SIZE * 1,
        0x03 => RAM_BANK_SIZE * 4,
        0x04 => RAM_BANK_SIZE * 16,
        0x05 => RAM_BANK_SIZE * 8,
        _ => panic!("Unused ram size!"),
    }
}

pub trait Cartridge {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, data: u8);
}

struct NoMBC {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl NoMBC {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = std::fs::read(path).unwrap();

        let rom_size = get_rom_size(rom[REGISTER_ROM_SIZE]);
        assert!(rom.len() == rom_size);

        let ram_size = get_ram_size(rom[REGISTER_RAM_SIZE]);
        let ram = vec![0; ram_size];

        NoMBC { rom, ram }
    }
}

impl Cartridge for NoMBC {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom[address],
            0xA000..=0xBFFF => self.ram[address - 0xA000],
            _ => panic!("Invalid address read!"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0xA000..=0xBFFF => self.ram[address - 0xA000] = data,
            _ => {} // ignore any other writes
        }
    }
}

struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    // registers
    enable_ram: bool,
    ram_banking_mode: bool,
    ram_bank: usize,
    rom_bank: usize,
}

impl MBC1 {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = std::fs::read(path).unwrap();

        let rom_size = get_rom_size(rom[REGISTER_ROM_SIZE]);
        assert!(rom.len() == rom_size);

        let ram_size = get_ram_size(rom[REGISTER_RAM_SIZE]);
        let ram = vec![0; ram_size];

        MBC1 {
            rom,
            ram,
            enable_ram: false,
            ram_banking_mode: false,
            ram_bank: 0,
            rom_bank: 1,
        }
    }
}

impl Cartridge for MBC1 {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => {
                let bank = self.rom_bank * ROM_BANK_SIZE;
                self.rom[bank + address - 0x4000]
            }
            0xA000..=0xBFFF => {
                let bank = self.ram_bank * RAM_BANK_SIZE;
                self.ram[bank + address - 0xA000]
            }
            _ => panic!("Invalid address read!"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0x0000..=0x1FFF => self.enable_ram = data & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                let data = if data == 0 { 1 } else { data };
                self.rom_bank = (self.rom_bank & 0b1110_0000) | (data as usize & 0b0001_1111);
            }
            0x4000..=0x5FFF => {
                if self.ram_banking_mode {
                    self.ram_bank = data as usize & 0x03;
                } else {
                    self.rom_bank =
                        (self.rom_bank & 0b0001_1111) | ((data as usize & 0b0000_0011) << 5);
                }
            }
            0x6000..=0x7FFF => self.ram_banking_mode = data & 0x01 == 0x01,
            0xA000..=0xBFFF => {
                if self.enable_ram {
                    let bank = self.ram_bank * RAM_BANK_SIZE;
                    self.ram[bank + address - 0xA000] = data;
                }
            }
            _ => {} // ignore any other writes
        }
    }
}

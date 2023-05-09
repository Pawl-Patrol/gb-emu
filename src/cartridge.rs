use crate::traits::{Memory, TestBit};
use std::fs::{create_dir_all, read, write, File};
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::Path;

pub trait Cartridge: Memory {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(&mut self, data: Vec<u8>);
}

const REGISTER_CARTRIDGE_TYPE: usize = 0x0147;
const REGISTER_ROM_SIZE: usize = 0x0148;
const REGISTER_RAM_SIZE: usize = 0x0149;
const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

pub fn load_rom(path: &str) -> Result<Box<dyn Cartridge>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(REGISTER_CARTRIDGE_TYPE as u64))?;
    let mut buffer = [0_u8; 1];
    file.read_exact(&mut buffer)?;
    println!("Cartridge type: {:#04X}", buffer[0]);
    match buffer[0] {
        0x00 | 0x08 | 0x09 => Ok(Box::new(NoMBC::load(path))),
        0x01 | 0x02 | 0x03 => Ok(Box::new(MBC1::load(path))),
        0x05 | 0x06 => Ok(Box::new(MBC2::load(path))),
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 => Ok(Box::new(MBC3::load(path))),
        0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => Ok(Box::new(MBC5::load(path))),
        _ => panic!("Unsupported cartridge type!"),
    }
}

pub fn save_state(cartridge: &Box<dyn Cartridge>, path: &str) -> Result<()> {
    let path = Path::new(path);
    let folder = path.parent().unwrap();
    if !folder.exists() {
        create_dir_all(folder)?;
    }
    let data = cartridge.serialize();
    write(path, data)?;
    Ok(())
}

pub fn load_state(cartridge: &mut Box<dyn Cartridge>, path: &str) -> Result<()> {
    let data = read(path)?;
    cartridge.deserialize(data);
    Ok(())
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

struct NoMBC {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl NoMBC {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = read(path).unwrap();
        let rom_size = get_rom_size(rom[REGISTER_ROM_SIZE]);
        assert!(rom.len() == rom_size);
        let ram_size = get_ram_size(rom[REGISTER_RAM_SIZE]);
        let ram = vec![0; ram_size];
        NoMBC { rom, ram }
    }
}

impl Memory for NoMBC {
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
            _ => {}
        }
    }
}

impl Cartridge for NoMBC {
    fn deserialize(&mut self, data: Vec<u8>) {
        self.ram = data;
    }

    fn serialize(&self) -> Vec<u8> {
        self.ram.clone()
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
        let rom: Vec<u8> = read(path).unwrap();
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

impl Memory for MBC1 {
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
            _ => {}
        }
    }
}

impl Cartridge for MBC1 {
    fn deserialize(&mut self, data: Vec<u8>) {
        self.ram = data;
    }

    fn serialize(&self) -> Vec<u8> {
        self.ram.clone()
    }
}

struct MBC2 {
    rom: Vec<u8>,
    ram: [u8; 256],
    // registers
    ram_enabled: bool,
    rom_bank: usize,
}

impl MBC2 {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = read(path).unwrap();
        let rom_size = get_rom_size(rom[REGISTER_ROM_SIZE]);
        assert!(rom.len() == rom_size);
        MBC2 {
            rom,
            ram: [0; 256],
            ram_enabled: false,
            rom_bank: 1,
        }
    }
}

impl Memory for MBC2 {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => {
                let bank = self.rom_bank * ROM_BANK_SIZE;
                self.rom[bank + address - 0x4000]
            }
            0xA000..=0xA1FF => self.ram[address - 0xA000],
            0xA200..=0xBFFF => self.ram[address - 0xA200], // echo ram
            _ => panic!("Invalid address read!"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0x0000..=0x3FFF => {
                if data.test_bit(7) {
                    let data = if data == 0 { 1 } else { data };
                    self.rom_bank = data as usize & 0b0000_1111;
                } else {
                    self.ram_enabled = data & 0x0F == 0x0A;
                }
            }
            0xA000..=0xA1FF => {
                if self.ram_enabled {
                    self.ram[address - 0xA000] = data & 0x0F;
                }
            }
            _ => {}
        }
    }
}

impl Cartridge for MBC2 {
    fn deserialize(&mut self, data: Vec<u8>) {
        self.ram = data.try_into().unwrap();
    }

    fn serialize(&self) -> Vec<u8> {
        self.ram.to_vec()
    }
}

struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    // registers
    ram_enabled: bool,
    ram_banking_mode: bool,
    ram_bank: usize,
    rom_bank: usize,
    rtc_select: usize,
    rtc: [u8; 5],
}

impl MBC3 {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = read(path).unwrap();
        let rom_size = get_rom_size(rom[REGISTER_ROM_SIZE]);
        assert!(rom.len() == rom_size);
        let ram_size = get_ram_size(rom[REGISTER_RAM_SIZE]);
        let ram = vec![0; ram_size];
        MBC3 {
            rom,
            ram,
            ram_enabled: false,
            ram_banking_mode: false,
            ram_bank: 0,
            rom_bank: 1,
            rtc_select: 0,
            rtc: [0; 5],
        }
    }
}

impl Memory for MBC3 {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => {
                let bank = self.rom_bank * ROM_BANK_SIZE;
                self.rom[bank + address - 0x4000]
            }
            0xA000..=0xBFFF => {
                if self.ram_banking_mode {
                    if self.ram_enabled {
                        let bank = self.ram_bank * RAM_BANK_SIZE;
                        self.ram[bank + address - 0xA000]
                    } else {
                        0xFF
                    }
                } else {
                    match self.rtc_select {
                        0x08 => self.rtc[0],
                        0x09 => self.rtc[1],
                        0x0A => self.rtc[2],
                        0x0B => self.rtc[3],
                        0x0C => self.rtc[4],
                        _ => panic!("Invalid RTC select!"),
                    }
                }
            }
            _ => panic!("Invalid address read!"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = data & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                let data = if data == 0 { 1 } else { data };
                self.rom_bank = data as usize & 0b0111_1111;
            }
            0x4000..=0x5FFF => {
                if data <= 0x03 {
                    self.ram_banking_mode = true;
                    self.ram_bank = data as usize;
                } else if data >= 0x08 && data <= 0x0C {
                    self.ram_banking_mode = false;
                    self.rtc_select = data as usize;
                }
            }
            0x6000..=0x7FFF => {} // Todo: Latch RTC
            0xA000..=0xBFFF => {
                if self.ram_banking_mode {
                    if self.ram_enabled {
                        let bank = self.ram_bank * RAM_BANK_SIZE;
                        self.ram[bank + address - 0xA000] = data;
                    }
                } else {
                    match self.rtc_select {
                        0x08 => self.rtc[0] = data,
                        0x09 => self.rtc[1] = data,
                        0x0A => self.rtc[2] = data,
                        0x0B => self.rtc[3] = data,
                        0x0C => self.rtc[4] = data,
                        _ => panic!("Invalid RTC select!"),
                    }
                }
            }
            _ => {}
        }
    }
}

impl Cartridge for MBC3 {
    fn deserialize(&mut self, data: Vec<u8>) {
        self.ram = data;
    }

    fn serialize(&self) -> Vec<u8> {
        self.ram.clone()
    }
}

struct MBC5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    // registers
    enable_ram: bool,
    ram_bank: usize,
    rom_bank: usize,
}

impl MBC5 {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = read(path).unwrap();
        let rom_size = get_rom_size(rom[REGISTER_ROM_SIZE]);
        assert!(rom.len() == rom_size);
        let ram_size = get_ram_size(rom[REGISTER_RAM_SIZE]);
        let ram = vec![0; ram_size];
        MBC5 {
            rom,
            ram,
            enable_ram: false,
            ram_bank: 0,
            rom_bank: 1,
        }
    }
}

impl Memory for MBC5 {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => {
                let bank = self.rom_bank * ROM_BANK_SIZE;
                self.rom[bank + address - 0x4000]
            }
            0xA000..=0xBFFF => {
                if self.enable_ram {
                    let bank = self.ram_bank * RAM_BANK_SIZE;
                    self.ram[bank + address - 0xA000]
                } else {
                    0xFF
                }
            }
            _ => panic!("Invalid address read!"),
        }
    }

    fn write(&mut self, address: usize, data: u8) {
        match address {
            0x0000..=0x1FFF => self.enable_ram = data & 0x0F == 0x0A,
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0b0001_0000_0000) | data as usize;
            }
            0x3000..=0x3FFF => {
                self.rom_bank =
                    (self.rom_bank & 0b0000_1111_1111) | ((data as usize & 0b0000_0001) << 8);
            }
            0x4000..=0x5FFF => {
                self.ram_bank = data as usize & 0x0F;
            }
            0xA000..=0xBFFF => {
                if self.enable_ram {
                    let bank = self.ram_bank * RAM_BANK_SIZE;
                    self.ram[bank + address - 0xA000] = data;
                }
            }
            _ => {}
        }
    }
}

impl Cartridge for MBC5 {
    fn deserialize(&mut self, data: Vec<u8>) {
        self.ram = data;
    }

    fn serialize(&self) -> Vec<u8> {
        self.ram.clone()
    }
}

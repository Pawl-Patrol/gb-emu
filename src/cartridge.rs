use std::io::{Read, Result, Seek};

pub fn load_rom(path: &str) -> Result<Box<dyn Cartridge>> {
    let mut file = std::fs::File::open(path)?;
    file.seek(std::io::SeekFrom::Start(0x0147))?;
    let mut buffer = [0_u8; 1];
    file.read_exact(&mut buffer)?;
    println!("Cartridge type: {:#04X}", buffer[0]);
    match buffer[0] {
        0x00 | 0x08 | 0x09 => Ok(Box::new(NoMBC::load(path))),
        _ => panic!("Unsupported cartridge type!"),
    }
}

fn get_rom_size(value_at_0x0148: u8) -> usize {
    match value_at_0x0148 {
        0x00 => 32 * 1024,
        0x01 => 64 * 1024,
        0x02 => 128 * 1024,
        0x03 => 256 * 1024,
        0x04 => 512 * 1024,
        0x05 => 1024 * 1024,
        0x06 => 2048 * 1024,
        0x07 => 4096 * 1024,
        0x08 => 8192 * 1024,
        _ => panic!("Unused rom size!"),
    }
}

fn get_ram_size(value_at_0x0149: u8) -> usize {
    match value_at_0x0149 {
        0x00 => 0,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => panic!("Unused ram size!"),
    }
}

pub trait Cartridge {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

struct NoMBC {
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
}

impl NoMBC {
    fn load(path: &str) -> Self {
        let rom: Vec<u8> = std::fs::read(path).unwrap();
        let rom_size = get_rom_size(rom[0x0148]);
        assert!(rom.len() == rom_size);
        match rom[0x0147] {
            0x00 => NoMBC { rom, ram: None },
            0x08 | 0x09 => {
                let ram = vec![0; get_ram_size(rom[0x0149])];
                NoMBC {
                    rom,
                    ram: Some(ram),
                }
            }
            _ => panic!("Cartridge type is not rom only!"),
        }
    }
}

impl Cartridge for NoMBC {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom[address as usize],
            0xA000..=0xBFFF => match &self.ram {
                Some(ram) => ram[(address - 0xA000) as usize],
                None => panic!("No cartridge ram!"),
            },
            _ => panic!("Invalid address read!"),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xA000..=0xBFFF => match &mut self.ram {
                Some(ram) => ram[(address - 0xA000) as usize] = data,
                None => panic!("No cartridge ram!"),
            },
            _ => {} // ignore any other writes
        }
    }
}

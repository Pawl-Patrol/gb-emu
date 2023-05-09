use crate::traits::Register;

// memory management unit
pub struct MMU {
    pub memory: [u8; 0x10000],
}

impl MMU {
    pub fn new() -> MMU {
        let mut memory = [0; 0x10000];
        memory[0xFF05] = 0x00;
        memory[0xFF06] = 0x00;
        memory[0xFF07] = 0x00;
        memory[0xFF10] = 0x80;
        memory[0xFF11] = 0xBF;
        memory[0xFF12] = 0xF3;
        memory[0xFF14] = 0xBF;
        memory[0xFF16] = 0x3F;
        memory[0xFF17] = 0x00;
        memory[0xFF19] = 0xBF;
        memory[0xFF1A] = 0x7F;
        memory[0xFF1B] = 0xFF;
        memory[0xFF1C] = 0x9F;
        memory[0xFF1E] = 0xBF;
        memory[0xFF20] = 0xFF;
        memory[0xFF21] = 0x00;
        memory[0xFF22] = 0x00;
        memory[0xFF23] = 0xBF;
        memory[0xFF24] = 0x77;
        memory[0xFF25] = 0xF3;
        memory[0xFF26] = 0xF1;
        memory[0xFF40] = 0x91;
        memory[0xFF42] = 0x00;
        memory[0xFF43] = 0x00;
        memory[0xFF45] = 0x00;
        memory[0xFF47] = 0xFC;
        memory[0xFF48] = 0xFF;
        memory[0xFF49] = 0xFF;
        memory[0xFF4A] = 0x00;
        memory[0xFF4B] = 0x00;
        memory[0xFFFF] = 0x00;
        MMU { memory }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn read_word(&self, address: u16) -> u16 {
        u16::from_bytes(
            self.memory[address as usize],
            self.memory[address as usize + 1],
        )
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.memory[address as usize] = value.hi();
        self.memory[address as usize + 1] = value.lo();
    }
}

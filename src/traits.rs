pub trait Cartridge {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, data: u8);
}

pub trait Register {
    fn hi(&self) -> u8;
    fn lo(&self) -> u8;
    fn set_lo(&mut self, lo: u8);
    fn set_hi(&mut self, hi: u8);
    fn from_bytes(hi: u8, lo: u8) -> Self;
}

impl Register for u16 {
    fn hi(&self) -> u8 {
        ((self & 0xFF00) >> 8) as u8
    }

    fn lo(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    fn set_lo(&mut self, lo: u8) {
        *self = (*self & 0xFF00) | (lo as u16);
    }

    fn set_hi(&mut self, hi: u8) {
        *self = (*self & 0x00FF) | ((hi as u16) << 8);
    }

    fn from_bytes(hi: u8, lo: u8) -> u16 {
        ((hi as u16) << 8) | (lo as u16)
    }
}

pub trait TestBit {
    fn test_bit(&self, bit: u8) -> bool;
}

impl TestBit for u8 {
    fn test_bit(&self, bit: u8) -> bool {
        (self & (1 << bit)) != 0
    }
}

impl TestBit for u16 {
    fn test_bit(&self, bit: u8) -> bool {
        (self & (1 << bit)) != 0
    }
}

pub trait SetBit {
    fn set_bit(&mut self, bit: u8);
    fn reset_bit(&mut self, bit: u8);
}

impl SetBit for u8 {
    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }

    fn reset_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }
}

pub trait ToggleBit {
    fn toggle_bit(&mut self, bit: u8, value: bool);
}

impl ToggleBit for u8 {
    fn toggle_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.set_bit(bit);
        } else {
            self.reset_bit(bit);
        }
    }
}

pub trait CarryTest {
    fn test_add_carry_bit(&self, value: Self, bit: u8) -> bool;
}

impl CarryTest for u16 {
    fn test_add_carry_bit(&self, value: u16, bit: u8) -> bool {
        let mask = (1_u16 << (bit + 1)) - 1_u16;
        (self & mask) + (value & mask) > mask
    }
}

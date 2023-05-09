use crate::{
    constants::{FLAG_CARRY, FLAG_HALF_CARRY, FLAG_SUBTRACT, FLAG_ZERO},
    emulator::Emulator,
    traits::{CarryTest, Register, SetBit, TestBit, ToggleBit},
};

// macro for loading 8-bit values into registers
macro_rules! cpu_8bit_load {
    ($emulator: ident, $register:ident) => {{
        $emulator.cpu.$register = $emulator.read_byte();
        8
    }};
}

macro_rules! cpu_16bit_load {
    ($emulator: ident, $hi: ident, $lo: ident, $word: expr) => {{
        let word = $word;
        $emulator.cpu.$hi = word.hi();
        $emulator.cpu.$lo = word.lo();
        12
    }};
}

macro_rules! cpu_reg_load {
    ($emulator: ident, $lhs: ident, $rhs: ident
    ) => {{
        $emulator.cpu.$lhs = $emulator.cpu.$rhs;
        4
    }};
}

macro_rules! cpu_reg_write {
    ($emulator: ident, $reg: ident, $addr: expr) => {{
        $emulator.write_memory($addr, $emulator.cpu.$reg);
        8
    }};
}

macro_rules! cpu_reg_load_rom {
    ($emulator: ident, $reg: ident, $addr: expr) => {{
        $emulator.cpu.$reg = $emulator.read_memory($addr);
        8
    }};
}

macro_rules! cpu_8bit_inc {
    ($emulator: ident, $reg: ident) => {{
        $emulator.cpu.$reg = $emulator.inc_8bit($emulator.cpu.$reg);
        4
    }};
}

macro_rules! cpu_8bit_dec {
    ($emulator: ident, $reg: ident) => {{
        $emulator.cpu.$reg = $emulator.dec_8bit($emulator.cpu.$reg);
        4
    }};
}

macro_rules! cpu_16bit_inc {
    ($emulator: ident, $hi: ident, $lo: ident) => {{
        let result = u16::from_bytes($emulator.cpu.$hi, $emulator.cpu.$lo).wrapping_add(1);
        $emulator.cpu.$hi = result.hi();
        $emulator.cpu.$lo = result.lo();
        8
    }};
}

macro_rules! cpu_16bit_dec {
    ($emulator: ident, $hi: ident, $lo: ident) => {{
        let result = u16::from_bytes($emulator.cpu.$hi, $emulator.cpu.$lo).wrapping_sub(1);
        $emulator.cpu.$hi = result.hi();
        $emulator.cpu.$lo = result.lo();
        8
    }};
}

macro_rules! rotate_left {
    ($emulator: ident, $reg: expr) => {{
        let ci = $emulator.cpu.f.test_bit(FLAG_CARRY) as u8;
        let co = $reg & 0x80;
        $reg = ($reg << 1) | ci;
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }};
}

macro_rules! rotate_left_carry {
    ($emulator: ident, $reg: expr) => {{
        let co = $reg & 0x80;
        $reg = $reg.rotate_left(1);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }};
}

macro_rules! rotate_right {
    ($emulator: ident, $reg: expr) => {{
        let ci = $emulator.cpu.f.test_bit(FLAG_CARRY) as u8;
        let co = $reg & 0x01;
        $reg = ($reg >> 1) | (ci << 7);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }};
}

macro_rules! rotate_right_carry {
    ($emulator: ident, $reg: expr) => {{
        let co = $reg & 0x01;
        $reg = $reg.rotate_right(1);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }};
}

macro_rules! rotate_left_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        rotate_left!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! rotate_right_carry_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        rotate_right!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! rotate_left_carry_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        rotate_left_carry!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! rotate_right_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        rotate_right_carry!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! shift_left_arithmetic {
    ($emulator: ident, $reg: expr) => {{
        let is_msb_set = $reg.test_bit(7);
        $reg <<= 1;
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, is_msb_set);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }};
}

macro_rules! shift_left_arithmetic_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        shift_left_arithmetic!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! shift_right_arithmetic {
    ($emulator: ident, $reg: expr) => {{
        let is_lsb_set = $reg.test_bit(0);
        let is_msb_set = $reg.test_bit(7);
        $reg >>= 1;
        $reg.toggle_bit(7, is_msb_set);
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, is_lsb_set);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }};
}

macro_rules! shift_right_arithmetic_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        shift_right_arithmetic!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! shift_right_logical {
    ($emulator: ident, $reg: expr) => {{
        let is_lsb_set = $reg.test_bit(0);
        $reg >>= 1;
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, is_lsb_set);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }};
}

macro_rules! shift_right_logical_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        shift_right_logical!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! swap_nibbles {
    ($emulator: ident, $reg: expr) => {{
        $reg = ($reg << 4) | ($reg >> 4);
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, $reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        $emulator.cpu.f.reset_bit(FLAG_CARRY);
        8
    }};
}

macro_rules! swap_nibbles_memory {
    ($emulator: ident, $addr: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        swap_nibbles!($emulator, reg);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! test_bit {
    ($emulator: ident, $reg: expr, $bit: expr) => {{
        $emulator.cpu.f.toggle_bit(FLAG_ZERO, !$reg.test_bit($bit));
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.set_bit(FLAG_HALF_CARRY);
        8
    }};
}

macro_rules! test_bit_memory {
    ($emulator: ident, $addr: expr, $bit: expr) => {{
        let reg = $emulator.read_memory($addr);
        test_bit!($emulator, reg, $bit);
        16
    }};
}

macro_rules! reset_bit {
    ($emulator: ident, $reg: expr, $bit: expr) => {{
        $reg.reset_bit($bit);
        8
    }};
}

macro_rules! reset_bit_memory {
    ($emulator: ident, $addr: expr, $bit: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        reset_bit!($emulator, reg, $bit);
        $emulator.write_memory($addr, reg);
        16
    }};
}

macro_rules! set_bit {
    ($emulator: ident, $reg: expr, $bit: expr) => {{
        $reg.set_bit($bit);
        8
    }};
}

macro_rules! set_bit_memory {
    ($emulator: ident, $addr: expr, $bit: expr) => {{
        let mut reg = $emulator.read_memory($addr);
        set_bit!($emulator, reg, $bit);
        $emulator.write_memory($addr, reg);
        16
    }};
}

impl Emulator {
    fn read_byte(&mut self) -> u8 {
        let result = self.read_memory(self.cpu.pc);
        self.cpu.pc += 1;
        result
    }

    fn read_word(&mut self) -> u16 {
        let lo = self.read_byte();
        let hi = self.read_byte();
        let word = u16::from_bytes(hi, lo);
        word
    }

    pub fn execute(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x00 => 4, // NOP

            // 8-bit loads
            0x06 => cpu_8bit_load!(self, b), // LD B, n
            0x0E => cpu_8bit_load!(self, c), // LD C, n
            0x16 => cpu_8bit_load!(self, d), // LD D, n
            0x1E => cpu_8bit_load!(self, e), // LD E, n
            0x26 => cpu_8bit_load!(self, h), // LD H, n
            0x2E => cpu_8bit_load!(self, l), // LD L,

            // // load register
            0x7F => cpu_reg_load!(self, a, a),
            0x78 => cpu_reg_load!(self, a, b),
            0x79 => cpu_reg_load!(self, a, c),
            0x7A => cpu_reg_load!(self, a, d),
            0x7B => cpu_reg_load!(self, a, e),
            0x7C => cpu_reg_load!(self, a, h),
            0x7D => cpu_reg_load!(self, a, l),
            0x40 => cpu_reg_load!(self, b, b),
            0x41 => cpu_reg_load!(self, b, c),
            0x42 => cpu_reg_load!(self, b, d),
            0x43 => cpu_reg_load!(self, b, e),
            0x44 => cpu_reg_load!(self, b, h),
            0x45 => cpu_reg_load!(self, b, l),
            0x48 => cpu_reg_load!(self, c, b),
            0x49 => cpu_reg_load!(self, c, c),
            0x4A => cpu_reg_load!(self, c, d),
            0x4B => cpu_reg_load!(self, c, e),
            0x4C => cpu_reg_load!(self, c, h),
            0x4D => cpu_reg_load!(self, c, l),
            0x50 => cpu_reg_load!(self, d, b),
            0x51 => cpu_reg_load!(self, d, c),
            0x52 => cpu_reg_load!(self, d, d),
            0x53 => cpu_reg_load!(self, d, e),
            0x54 => cpu_reg_load!(self, d, h),
            0x55 => cpu_reg_load!(self, d, l),
            0x58 => cpu_reg_load!(self, e, b),
            0x59 => cpu_reg_load!(self, e, c),
            0x5A => cpu_reg_load!(self, e, d),
            0x5B => cpu_reg_load!(self, e, e),
            0x5C => cpu_reg_load!(self, e, h),
            0x5D => cpu_reg_load!(self, e, l),
            0x60 => cpu_reg_load!(self, h, b),
            0x61 => cpu_reg_load!(self, h, c),
            0x62 => cpu_reg_load!(self, h, d),
            0x63 => cpu_reg_load!(self, h, e),
            0x64 => cpu_reg_load!(self, h, h),
            0x65 => cpu_reg_load!(self, h, l),
            0x68 => cpu_reg_load!(self, l, b),
            0x69 => cpu_reg_load!(self, l, c),
            0x6A => cpu_reg_load!(self, l, d),
            0x6B => cpu_reg_load!(self, l, e),
            0x6C => cpu_reg_load!(self, l, h),
            0x6D => cpu_reg_load!(self, l, l),

            // // write register to memory
            0x70 => cpu_reg_write!(self, b, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x71 => cpu_reg_write!(self, c, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x72 => cpu_reg_write!(self, d, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x73 => cpu_reg_write!(self, e, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x74 => cpu_reg_write!(self, h, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x75 => cpu_reg_write!(self, l, u16::from_bytes(self.cpu.h, self.cpu.l)),

            // // write memory to register
            0x7E => cpu_reg_load_rom!(self, a, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x46 => cpu_reg_load_rom!(self, b, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x4E => cpu_reg_load_rom!(self, c, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x56 => cpu_reg_load_rom!(self, d, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x5E => cpu_reg_load_rom!(self, e, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x66 => cpu_reg_load_rom!(self, h, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x6E => cpu_reg_load_rom!(self, l, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x0A => cpu_reg_load_rom!(self, a, u16::from_bytes(self.cpu.b, self.cpu.c)),
            0x1A => cpu_reg_load_rom!(self, a, u16::from_bytes(self.cpu.d, self.cpu.e)),
            0xF2 => cpu_reg_load_rom!(self, a, u16::from_bytes(0xFF, self.cpu.c)),

            // // put a into register
            0x47 => cpu_reg_load!(self, b, a),
            0x4F => cpu_reg_load!(self, c, a),
            0x57 => cpu_reg_load!(self, d, a),
            0x5F => cpu_reg_load!(self, e, a),
            0x67 => cpu_reg_load!(self, h, a),
            0x6F => cpu_reg_load!(self, l, a),

            // put a into memory address
            0x02 => cpu_reg_write!(self, a, u16::from_bytes(self.cpu.b, self.cpu.c)),
            0x12 => cpu_reg_write!(self, a, u16::from_bytes(self.cpu.d, self.cpu.e)),
            0x77 => cpu_reg_write!(self, a, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0xE2 => cpu_reg_write!(self, a, u16::from_bytes(0xff, self.cpu.c)),

            // // put memory into a, decrement/increment HL
            0x3A => {
                cpu_reg_load_rom!(self, a, u16::from_bytes(self.cpu.h, self.cpu.l))
                    + cpu_16bit_dec!(self, h, l)
            }
            0x2A => {
                cpu_reg_load_rom!(self, a, u16::from_bytes(self.cpu.h, self.cpu.l))
                    + cpu_16bit_inc!(self, h, l)
            }

            // // put a into memory, decrement/increment memory
            0x32 => {
                cpu_reg_write!(self, a, u16::from_bytes(self.cpu.h, self.cpu.l))
                    + cpu_16bit_dec!(self, h, l)
            }
            0x22 => {
                cpu_reg_write!(self, a, u16::from_bytes(self.cpu.h, self.cpu.l))
                    + cpu_16bit_inc!(self, h, l)
            }

            // // 16 bit loads
            0x01 => cpu_16bit_load!(self, b, c, self.read_word()),
            0x11 => cpu_16bit_load!(self, d, e, self.read_word()),
            0x21 => cpu_16bit_load!(self, h, l, self.read_word()),
            0x31 => {
                self.cpu.sp = self.read_word();
                12
            }
            0xF9 => {
                self.cpu.sp = u16::from_bytes(self.cpu.h, self.cpu.l);
                8
            }

            // // push word onto stack
            0xF5 => {
                self.push_stack(u16::from_bytes(self.cpu.a, self.cpu.f));
                16
            }
            0xC5 => {
                self.push_stack(u16::from_bytes(self.cpu.b, self.cpu.c));
                16
            }
            0xD5 => {
                self.push_stack(u16::from_bytes(self.cpu.d, self.cpu.e));
                16
            }
            0xE5 => {
                self.push_stack(u16::from_bytes(self.cpu.h, self.cpu.l));
                16
            }

            // // pop word from stack into register
            0xF1 => {
                let word = self.pop_stack();
                self.cpu.a = word.hi();
                self.cpu.f = word.lo() & 0xF0;
                12
            }
            0xC1 => cpu_16bit_load!(self, b, c, self.pop_stack()),
            0xD1 => cpu_16bit_load!(self, d, e, self.pop_stack()),
            0xE1 => cpu_16bit_load!(self, h, l, self.pop_stack()),

            // 8-bit add
            0x87 => self.add_8bit(Some(self.cpu.a)),
            0x80 => self.add_8bit(Some(self.cpu.b)),
            0x81 => self.add_8bit(Some(self.cpu.c)),
            0x82 => self.add_8bit(Some(self.cpu.d)),
            0x83 => self.add_8bit(Some(self.cpu.e)),
            0x84 => self.add_8bit(Some(self.cpu.h)),
            0x85 => self.add_8bit(Some(self.cpu.l)),
            0x86 => {
                self.add_8bit(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xC6 => self.add_8bit(None) + 4,

            // 8-bit add + carry
            0x8F => self.add_8bit_carry(Some(self.cpu.a)),
            0x88 => self.add_8bit_carry(Some(self.cpu.b)),
            0x89 => self.add_8bit_carry(Some(self.cpu.c)),
            0x8A => self.add_8bit_carry(Some(self.cpu.d)),
            0x8B => self.add_8bit_carry(Some(self.cpu.e)),
            0x8C => self.add_8bit_carry(Some(self.cpu.h)),
            0x8D => self.add_8bit_carry(Some(self.cpu.l)),
            0x8E => {
                self.add_8bit_carry(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xCE => self.add_8bit_carry(None) + 4,

            // 8-bit subtract
            0x97 => self.sub_8bit(Some(self.cpu.a)),
            0x90 => self.sub_8bit(Some(self.cpu.b)),
            0x91 => self.sub_8bit(Some(self.cpu.c)),
            0x92 => self.sub_8bit(Some(self.cpu.d)),
            0x93 => self.sub_8bit(Some(self.cpu.e)),
            0x94 => self.sub_8bit(Some(self.cpu.h)),
            0x95 => self.sub_8bit(Some(self.cpu.l)),
            0x96 => {
                self.sub_8bit(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xD6 => self.sub_8bit(None) + 4,

            // 8-bit subtract + carry
            0x9F => self.sub_8bit_carry(Some(self.cpu.a)),
            0x98 => self.sub_8bit_carry(Some(self.cpu.b)),
            0x99 => self.sub_8bit_carry(Some(self.cpu.c)),
            0x9A => self.sub_8bit_carry(Some(self.cpu.d)),
            0x9B => self.sub_8bit_carry(Some(self.cpu.e)),
            0x9C => self.sub_8bit_carry(Some(self.cpu.h)),
            0x9D => self.sub_8bit_carry(Some(self.cpu.l)),
            0x9E => {
                self.sub_8bit_carry(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xDE => self.sub_8bit_carry(None) + 4,

            // 8-bit AND
            0xA7 => self.and_8bit(Some(self.cpu.a)),
            0xA0 => self.and_8bit(Some(self.cpu.b)),
            0xA1 => self.and_8bit(Some(self.cpu.c)),
            0xA2 => self.and_8bit(Some(self.cpu.d)),
            0xA3 => self.and_8bit(Some(self.cpu.e)),
            0xA4 => self.and_8bit(Some(self.cpu.h)),
            0xA5 => self.and_8bit(Some(self.cpu.l)),
            0xA6 => {
                self.and_8bit(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xE6 => self.and_8bit(None) + 4,

            // 8-bit OR
            0xB7 => self.or_8bit(Some(self.cpu.a)),
            0xB0 => self.or_8bit(Some(self.cpu.b)),
            0xB1 => self.or_8bit(Some(self.cpu.c)),
            0xB2 => self.or_8bit(Some(self.cpu.d)),
            0xB3 => self.or_8bit(Some(self.cpu.e)),
            0xB4 => self.or_8bit(Some(self.cpu.h)),
            0xB5 => self.or_8bit(Some(self.cpu.l)),
            0xB6 => {
                self.or_8bit(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xF6 => self.or_8bit(None) + 4,

            // 8-bit XOR
            0xAF => self.xor_8bit(Some(self.cpu.a)),
            0xA8 => self.xor_8bit(Some(self.cpu.b)),
            0xA9 => self.xor_8bit(Some(self.cpu.c)),
            0xAA => self.xor_8bit(Some(self.cpu.d)),
            0xAB => self.xor_8bit(Some(self.cpu.e)),
            0xAC => self.xor_8bit(Some(self.cpu.h)),
            0xAD => self.xor_8bit(Some(self.cpu.l)),
            0xAE => {
                self.xor_8bit(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xEE => self.xor_8bit(None) + 4,

            // 8-bit compare
            0xBF => self.compare_8bit(Some(self.cpu.a)),
            0xB8 => self.compare_8bit(Some(self.cpu.b)),
            0xB9 => self.compare_8bit(Some(self.cpu.c)),
            0xBA => self.compare_8bit(Some(self.cpu.d)),
            0xBB => self.compare_8bit(Some(self.cpu.e)),
            0xBC => self.compare_8bit(Some(self.cpu.h)),
            0xBD => self.compare_8bit(Some(self.cpu.l)),
            0xBE => {
                self.compare_8bit(Some(
                    self.read_memory(u16::from_bytes(self.cpu.h, self.cpu.l)),
                )) + 4
            }
            0xFE => self.compare_8bit(None) + 4,

            // 8-bit increment
            0x3C => cpu_8bit_inc!(self, a),
            0x04 => cpu_8bit_inc!(self, b),
            0x0C => cpu_8bit_inc!(self, c),
            0x14 => cpu_8bit_inc!(self, d),
            0x1C => cpu_8bit_inc!(self, e),
            0x24 => cpu_8bit_inc!(self, h),
            0x2C => cpu_8bit_inc!(self, l),
            0x34 => {
                let hl = u16::from_bytes(self.cpu.h, self.cpu.l);
                let value = self.read_memory(hl);
                let result = value.wrapping_add(1);
                self.write_memory(hl, result);
                self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
                self.cpu.f.reset_bit(FLAG_SUBTRACT);
                self.cpu.f.toggle_bit(FLAG_HALF_CARRY, value & 0x0f == 0x0f);
                12
            }

            // 8-bit decrement
            0x3D => cpu_8bit_dec!(self, a),
            0x05 => cpu_8bit_dec!(self, b),
            0x0D => cpu_8bit_dec!(self, c),
            0x15 => cpu_8bit_dec!(self, d),
            0x1D => cpu_8bit_dec!(self, e),
            0x25 => cpu_8bit_dec!(self, h),
            0x2D => cpu_8bit_dec!(self, l),
            0x35 => {
                let hl = u16::from_bytes(self.cpu.h, self.cpu.l);
                let value = self.read_memory(hl);
                let result = value.wrapping_sub(1);
                self.write_memory(hl, result);
                self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
                self.cpu.f.set_bit(FLAG_SUBTRACT);
                self.cpu.f.toggle_bit(FLAG_HALF_CARRY, value & 0x0f == 0x00);
                12
            }

            // 16-bit add
            0x09 => self.add_16bit(u16::from_bytes(self.cpu.b, self.cpu.c)),
            0x19 => self.add_16bit(u16::from_bytes(self.cpu.d, self.cpu.e)),
            0x29 => self.add_16bit(u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x39 => self.add_16bit(self.cpu.sp),

            // 16-bit increment
            0x03 => cpu_16bit_inc!(self, b, c),
            0x13 => cpu_16bit_inc!(self, d, e),
            0x23 => cpu_16bit_inc!(self, h, l),
            0x33 => {
                self.cpu.sp = self.cpu.sp.wrapping_add(1);
                8
            }

            // 16-bit decrement
            0x0B => cpu_16bit_dec!(self, b, c),
            0x1B => cpu_16bit_dec!(self, d, e),
            0x2B => cpu_16bit_dec!(self, h, l),
            0x3B => {
                self.cpu.sp = self.cpu.sp.wrapping_sub(1);
                8
            }

            // jumps
            0xE9 => {
                self.cpu.pc = u16::from_bytes(self.cpu.h, self.cpu.l);
                4
            }
            0xC3 => self.jump(0, false, false),
            0xC2 => self.jump(FLAG_ZERO, true, false),
            0xCA => self.jump(FLAG_ZERO, true, true),
            0xD2 => self.jump(FLAG_CARRY, true, false),
            0xDA => self.jump(FLAG_CARRY, true, true),

            // jump with immediate data
            0x18 => self.jump_immediate(0, false, false),
            0x20 => self.jump_immediate(FLAG_ZERO, true, false),
            0x28 => self.jump_immediate(FLAG_ZERO, true, true),
            0x30 => self.jump_immediate(FLAG_CARRY, true, false),
            0x38 => self.jump_immediate(FLAG_CARRY, true, true),

            // call
            0xCD => self.call(0, false, false),
            0xC4 => self.call(FLAG_ZERO, true, false),
            0xCC => self.call(FLAG_ZERO, true, true),
            0xD4 => self.call(FLAG_CARRY, true, false),
            0xDC => self.call(FLAG_CARRY, true, true),

            // return
            0xC9 => self.return_from_call(0, false, false),
            0xC0 => self.return_from_call(FLAG_ZERO, true, false),
            0xC8 => self.return_from_call(FLAG_ZERO, true, true),
            0xD0 => self.return_from_call(FLAG_CARRY, true, false),
            0xD8 => self.return_from_call(FLAG_CARRY, true, true),

            // restart
            0xC7 => self.restart(0x00),
            0xCF => self.restart(0x08),
            0xD7 => self.restart(0x10),
            0xDF => self.restart(0x18),
            0xE7 => self.restart(0x20),
            0xEF => self.restart(0x28),
            0xF7 => self.restart(0x30),
            0xFF => self.restart(0x38),

            // decimal adjust register A
            0x27 => {
                let mut carry = false;
                if !self.cpu.f.test_bit(FLAG_SUBTRACT) {
                    if self.cpu.f.test_bit(FLAG_CARRY) || self.cpu.a > 0x99 {
                        self.cpu.a = self.cpu.a.wrapping_add(0x60);
                        carry = true;
                    }
                    if self.cpu.f.test_bit(FLAG_HALF_CARRY) || self.cpu.a & 0x0f > 0x09 {
                        self.cpu.a = self.cpu.a.wrapping_add(0x06);
                    }
                } else if self.cpu.f.test_bit(FLAG_CARRY) {
                    carry = true;
                    let adder = if self.cpu.f.test_bit(FLAG_HALF_CARRY) {
                        0x9a
                    } else {
                        0xa0
                    };
                    self.cpu.a = self.cpu.a.wrapping_add(adder);
                } else if self.cpu.f.test_bit(FLAG_HALF_CARRY) {
                    self.cpu.a = self.cpu.a.wrapping_add(0xfa);
                }
                self.cpu.f.toggle_bit(FLAG_ZERO, self.cpu.a == 0);
                self.cpu.f.reset_bit(FLAG_HALF_CARRY);
                self.cpu.f.toggle_bit(FLAG_CARRY, carry);
                4
            }

            0xCB => self.execute_extended(),

            0x07 => {
                let cylces = rotate_left_carry!(self, self.cpu.a);
                self.cpu.f.reset_bit(FLAG_ZERO);
                cylces
            }
            0x0F => {
                let cycles = rotate_right_carry!(self, self.cpu.a);
                self.cpu.f.reset_bit(FLAG_ZERO);
                cycles
            }
            0x17 => {
                let cycles = rotate_left!(self, self.cpu.a);
                self.cpu.f.reset_bit(FLAG_ZERO);
                cycles
            }
            0x1F => {
                let cycles = rotate_right!(self, self.cpu.a);
                self.cpu.f.reset_bit(FLAG_ZERO);
                cycles
            }

            0xD9 => {
                self.cpu.pc = self.pop_stack();
                self.interrupts_enabled = true;
                8
            }

            0x08 => {
                let address = self.read_word();
                self.write_memory(address, self.cpu.sp.lo());
                self.write_memory(address.wrapping_add(1), self.cpu.sp.hi());
                20
            }

            0x36 => {
                let byte = self.read_byte();
                self.write_memory(u16::from_bytes(self.cpu.h, self.cpu.l), byte);
                12
            }

            0xFA => {
                let address = self.read_word();
                self.cpu.a = self.read_memory(address);
                16
            }

            0x3E => {
                self.cpu.a = self.read_byte();
                8
            }

            0xEA => {
                let address = self.read_word();
                self.write_memory(address, self.cpu.a);
                16
            }

            // disable interrupts
            0xF3 => {
                self.pending_interrupt = Some(false);
                4
            }

            // enable interrupts
            0xFB => {
                self.pending_interrupt = Some(true);
                4
            }

            0xE0 => {
                let address = u16::from_bytes(0xFF, self.read_byte());
                self.write_memory(address, self.cpu.a);
                12
            }

            0xF0 => {
                let address = u16::from_bytes(0xFF, self.read_byte());
                self.cpu.a = self.read_memory(address);
                12
            }

            0x2F => {
                self.cpu.a ^= 0xFF;
                self.cpu.f.set_bit(FLAG_SUBTRACT);
                self.cpu.f.set_bit(FLAG_HALF_CARRY);
                4
            }

            0x76 => {
                self.halted = true;
                4
            }

            0x3F => {
                self.cpu
                    .f
                    .toggle_bit(FLAG_CARRY, !self.cpu.f.test_bit(FLAG_CARRY));
                self.cpu.f.reset_bit(FLAG_SUBTRACT);
                self.cpu.f.reset_bit(FLAG_HALF_CARRY);
                4
            }

            0x37 => {
                self.cpu.f.set_bit(FLAG_CARRY);
                self.cpu.f.reset_bit(FLAG_SUBTRACT);
                self.cpu.f.reset_bit(FLAG_HALF_CARRY);
                4
            }

            0xF8 => {
                let offset = self.read_byte() as i8 as i16 as u16;
                let result = self.cpu.sp.wrapping_add(offset);
                self.cpu.f.reset_bit(FLAG_ZERO);
                self.cpu.f.reset_bit(FLAG_SUBTRACT);
                self.cpu
                    .f
                    .toggle_bit(FLAG_HALF_CARRY, self.cpu.sp.test_add_carry_bit(offset, 3));
                self.cpu
                    .f
                    .toggle_bit(FLAG_CARRY, self.cpu.sp.test_add_carry_bit(offset, 7));
                self.cpu.h = result.hi();
                self.cpu.l = result.lo();
                12
            }

            0x10 => {
                self.cpu.pc += 1;
                4
            }

            0xe8 => {
                let offset = self.read_byte() as i8 as i16 as u16;
                let result = self.cpu.sp.wrapping_add(offset);
                self.cpu.f.reset_bit(FLAG_ZERO);
                self.cpu.f.reset_bit(FLAG_SUBTRACT);
                self.cpu
                    .f
                    .toggle_bit(FLAG_HALF_CARRY, self.cpu.sp.test_add_carry_bit(offset, 3));
                self.cpu
                    .f
                    .toggle_bit(FLAG_CARRY, self.cpu.sp.test_add_carry_bit(offset, 7));
                self.cpu.sp = result;
                16
            }

            _ => panic!("Unknown opcode: {:02X}", opcode),
        }
    }

    fn execute_extended(&mut self) -> u32 {
        let opcode = self.read_byte();

        match opcode {
            // rotate left through carry
            0x0 => rotate_left_carry!(self, self.cpu.b),
            0x1 => rotate_left_carry!(self, self.cpu.c),
            0x2 => rotate_left_carry!(self, self.cpu.d),
            0x3 => rotate_left_carry!(self, self.cpu.e),
            0x4 => rotate_left_carry!(self, self.cpu.h),
            0x5 => rotate_left_carry!(self, self.cpu.l),
            0x6 => rotate_left_carry_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x7 => rotate_left_carry!(self, self.cpu.a),

            // rotate right through carry
            0x8 => rotate_right_carry!(self, self.cpu.b),
            0x9 => rotate_right_carry!(self, self.cpu.c),
            0xA => rotate_right_carry!(self, self.cpu.d),
            0xB => rotate_right_carry!(self, self.cpu.e),
            0xC => rotate_right_carry!(self, self.cpu.h),
            0xD => rotate_right_carry!(self, self.cpu.l),
            0xE => rotate_right_carry_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0xF => rotate_right_carry!(self, self.cpu.a),

            // rotate left
            0x10 => rotate_left!(self, self.cpu.b),
            0x11 => rotate_left!(self, self.cpu.c),
            0x12 => rotate_left!(self, self.cpu.d),
            0x13 => rotate_left!(self, self.cpu.e),
            0x14 => rotate_left!(self, self.cpu.h),
            0x15 => rotate_left!(self, self.cpu.l),
            0x16 => rotate_left_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x17 => rotate_left!(self, self.cpu.a),

            // rotate right
            0x18 => rotate_right!(self, self.cpu.b),
            0x19 => rotate_right!(self, self.cpu.c),
            0x1A => rotate_right!(self, self.cpu.d),
            0x1B => rotate_right!(self, self.cpu.e),
            0x1C => rotate_right!(self, self.cpu.h),
            0x1D => rotate_right!(self, self.cpu.l),
            0x1E => rotate_right_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x1F => rotate_right!(self, self.cpu.a),

            // shift left arithmetic
            0x20 => shift_left_arithmetic!(self, self.cpu.b),
            0x21 => shift_left_arithmetic!(self, self.cpu.c),
            0x22 => shift_left_arithmetic!(self, self.cpu.d),
            0x23 => shift_left_arithmetic!(self, self.cpu.e),
            0x24 => shift_left_arithmetic!(self, self.cpu.h),
            0x25 => shift_left_arithmetic!(self, self.cpu.l),
            0x26 => shift_left_arithmetic_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x27 => shift_left_arithmetic!(self, self.cpu.a),

            // shift right arithmetic
            0x28 => shift_right_arithmetic!(self, self.cpu.b),
            0x29 => shift_right_arithmetic!(self, self.cpu.c),
            0x2A => shift_right_arithmetic!(self, self.cpu.d),
            0x2B => shift_right_arithmetic!(self, self.cpu.e),
            0x2C => shift_right_arithmetic!(self, self.cpu.h),
            0x2D => shift_right_arithmetic!(self, self.cpu.l),
            0x2E => shift_right_arithmetic_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x2F => shift_right_arithmetic!(self, self.cpu.a),

            // shift right logical
            0x38 => shift_right_logical!(self, self.cpu.b),
            0x39 => shift_right_logical!(self, self.cpu.c),
            0x3A => shift_right_logical!(self, self.cpu.d),
            0x3B => shift_right_logical!(self, self.cpu.e),
            0x3C => shift_right_logical!(self, self.cpu.h),
            0x3D => shift_right_logical!(self, self.cpu.l),
            0x3E => shift_right_logical_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),
            0x3F => shift_right_logical!(self, self.cpu.a),

            // swap nibbles
            0x37 => swap_nibbles!(self, self.cpu.a),
            0x30 => swap_nibbles!(self, self.cpu.b),
            0x31 => swap_nibbles!(self, self.cpu.c),
            0x32 => swap_nibbles!(self, self.cpu.d),
            0x33 => swap_nibbles!(self, self.cpu.e),
            0x34 => swap_nibbles!(self, self.cpu.h),
            0x35 => swap_nibbles!(self, self.cpu.l),
            0x36 => swap_nibbles_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l)),

            // test bit 0
            0x40 => test_bit!(self, self.cpu.b, 0),
            0x41 => test_bit!(self, self.cpu.c, 0),
            0x42 => test_bit!(self, self.cpu.d, 0),
            0x43 => test_bit!(self, self.cpu.e, 0),
            0x44 => test_bit!(self, self.cpu.h, 0),
            0x45 => test_bit!(self, self.cpu.l, 0),
            0x46 => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 0),
            0x47 => test_bit!(self, self.cpu.a, 0),

            // test bit 1
            0x48 => test_bit!(self, self.cpu.b, 1),
            0x49 => test_bit!(self, self.cpu.c, 1),
            0x4A => test_bit!(self, self.cpu.d, 1),
            0x4B => test_bit!(self, self.cpu.e, 1),
            0x4C => test_bit!(self, self.cpu.h, 1),
            0x4D => test_bit!(self, self.cpu.l, 1),
            0x4E => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 1),
            0x4F => test_bit!(self, self.cpu.a, 1),

            // test bit 2
            0x50 => test_bit!(self, self.cpu.b, 2),
            0x51 => test_bit!(self, self.cpu.c, 2),
            0x52 => test_bit!(self, self.cpu.d, 2),
            0x53 => test_bit!(self, self.cpu.e, 2),
            0x54 => test_bit!(self, self.cpu.h, 2),
            0x55 => test_bit!(self, self.cpu.l, 2),
            0x56 => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 2),
            0x57 => test_bit!(self, self.cpu.a, 2),

            // test bit 3
            0x58 => test_bit!(self, self.cpu.b, 3),
            0x59 => test_bit!(self, self.cpu.c, 3),
            0x5A => test_bit!(self, self.cpu.d, 3),
            0x5B => test_bit!(self, self.cpu.e, 3),
            0x5C => test_bit!(self, self.cpu.h, 3),
            0x5D => test_bit!(self, self.cpu.l, 3),
            0x5E => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 3),
            0x5F => test_bit!(self, self.cpu.a, 3),

            // test bit 4
            0x60 => test_bit!(self, self.cpu.b, 4),
            0x61 => test_bit!(self, self.cpu.c, 4),
            0x62 => test_bit!(self, self.cpu.d, 4),
            0x63 => test_bit!(self, self.cpu.e, 4),
            0x64 => test_bit!(self, self.cpu.h, 4),
            0x65 => test_bit!(self, self.cpu.l, 4),
            0x66 => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 4),
            0x67 => test_bit!(self, self.cpu.a, 4),

            // test bit 5
            0x68 => test_bit!(self, self.cpu.b, 5),
            0x69 => test_bit!(self, self.cpu.c, 5),
            0x6A => test_bit!(self, self.cpu.d, 5),
            0x6B => test_bit!(self, self.cpu.e, 5),
            0x6C => test_bit!(self, self.cpu.h, 5),
            0x6D => test_bit!(self, self.cpu.l, 5),
            0x6E => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 5),
            0x6F => test_bit!(self, self.cpu.a, 5),

            // test bit 6
            0x70 => test_bit!(self, self.cpu.b, 6),
            0x71 => test_bit!(self, self.cpu.c, 6),
            0x72 => test_bit!(self, self.cpu.d, 6),
            0x73 => test_bit!(self, self.cpu.e, 6),
            0x74 => test_bit!(self, self.cpu.h, 6),
            0x75 => test_bit!(self, self.cpu.l, 6),
            0x76 => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 6),
            0x77 => test_bit!(self, self.cpu.a, 6),

            // test bit 7
            0x78 => test_bit!(self, self.cpu.b, 7),
            0x79 => test_bit!(self, self.cpu.c, 7),
            0x7A => test_bit!(self, self.cpu.d, 7),
            0x7B => test_bit!(self, self.cpu.e, 7),
            0x7C => test_bit!(self, self.cpu.h, 7),
            0x7D => test_bit!(self, self.cpu.l, 7),
            0x7E => test_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 7),
            0x7F => test_bit!(self, self.cpu.a, 7),

            // reset bit 0
            0x80 => reset_bit!(self, self.cpu.b, 0),
            0x81 => reset_bit!(self, self.cpu.c, 0),
            0x82 => reset_bit!(self, self.cpu.d, 0),
            0x83 => reset_bit!(self, self.cpu.e, 0),
            0x84 => reset_bit!(self, self.cpu.h, 0),
            0x85 => reset_bit!(self, self.cpu.l, 0),
            0x86 => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 0),
            0x87 => reset_bit!(self, self.cpu.a, 0),

            // reset bit 1
            0x88 => reset_bit!(self, self.cpu.b, 1),
            0x89 => reset_bit!(self, self.cpu.c, 1),
            0x8A => reset_bit!(self, self.cpu.d, 1),
            0x8B => reset_bit!(self, self.cpu.e, 1),
            0x8C => reset_bit!(self, self.cpu.h, 1),
            0x8D => reset_bit!(self, self.cpu.l, 1),
            0x8E => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 1),
            0x8F => reset_bit!(self, self.cpu.a, 1),

            // reset bit 2
            0x90 => reset_bit!(self, self.cpu.b, 2),
            0x91 => reset_bit!(self, self.cpu.c, 2),
            0x92 => reset_bit!(self, self.cpu.d, 2),
            0x93 => reset_bit!(self, self.cpu.e, 2),
            0x94 => reset_bit!(self, self.cpu.h, 2),
            0x95 => reset_bit!(self, self.cpu.l, 2),
            0x96 => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 2),
            0x97 => reset_bit!(self, self.cpu.a, 2),

            // reset bit 3
            0x98 => reset_bit!(self, self.cpu.b, 3),
            0x99 => reset_bit!(self, self.cpu.c, 3),
            0x9A => reset_bit!(self, self.cpu.d, 3),
            0x9B => reset_bit!(self, self.cpu.e, 3),
            0x9C => reset_bit!(self, self.cpu.h, 3),
            0x9D => reset_bit!(self, self.cpu.l, 3),
            0x9E => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 3),
            0x9F => reset_bit!(self, self.cpu.a, 3),

            // reset bit 4
            0xA0 => reset_bit!(self, self.cpu.b, 4),
            0xA1 => reset_bit!(self, self.cpu.c, 4),
            0xA2 => reset_bit!(self, self.cpu.d, 4),
            0xA3 => reset_bit!(self, self.cpu.e, 4),
            0xA4 => reset_bit!(self, self.cpu.h, 4),
            0xA5 => reset_bit!(self, self.cpu.l, 4),
            0xA6 => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 4),
            0xA7 => reset_bit!(self, self.cpu.a, 4),

            // reset bit 5
            0xA8 => reset_bit!(self, self.cpu.b, 5),
            0xA9 => reset_bit!(self, self.cpu.c, 5),
            0xAA => reset_bit!(self, self.cpu.d, 5),
            0xAB => reset_bit!(self, self.cpu.e, 5),
            0xAC => reset_bit!(self, self.cpu.h, 5),
            0xAD => reset_bit!(self, self.cpu.l, 5),
            0xAE => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 5),
            0xAF => reset_bit!(self, self.cpu.a, 5),

            // reset bit 6
            0xB0 => reset_bit!(self, self.cpu.b, 6),
            0xB1 => reset_bit!(self, self.cpu.c, 6),
            0xB2 => reset_bit!(self, self.cpu.d, 6),
            0xB3 => reset_bit!(self, self.cpu.e, 6),
            0xB4 => reset_bit!(self, self.cpu.h, 6),
            0xB5 => reset_bit!(self, self.cpu.l, 6),
            0xB6 => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 6),
            0xB7 => reset_bit!(self, self.cpu.a, 6),

            // reset bit 7
            0xB8 => reset_bit!(self, self.cpu.b, 7),
            0xB9 => reset_bit!(self, self.cpu.c, 7),
            0xBA => reset_bit!(self, self.cpu.d, 7),
            0xBB => reset_bit!(self, self.cpu.e, 7),
            0xBC => reset_bit!(self, self.cpu.h, 7),
            0xBD => reset_bit!(self, self.cpu.l, 7),
            0xBE => reset_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 7),
            0xBF => reset_bit!(self, self.cpu.a, 7),

            // set bit 0
            0xC0 => set_bit!(self, self.cpu.b, 0),
            0xC1 => set_bit!(self, self.cpu.c, 0),
            0xC2 => set_bit!(self, self.cpu.d, 0),
            0xC3 => set_bit!(self, self.cpu.e, 0),
            0xC4 => set_bit!(self, self.cpu.h, 0),
            0xC5 => set_bit!(self, self.cpu.l, 0),
            0xC6 => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 0),
            0xC7 => set_bit!(self, self.cpu.a, 0),

            // set bit 1
            0xC8 => set_bit!(self, self.cpu.b, 1),
            0xC9 => set_bit!(self, self.cpu.c, 1),
            0xCA => set_bit!(self, self.cpu.d, 1),
            0xCB => set_bit!(self, self.cpu.e, 1),
            0xCC => set_bit!(self, self.cpu.h, 1),
            0xCD => set_bit!(self, self.cpu.l, 1),
            0xCE => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 1),
            0xCF => set_bit!(self, self.cpu.a, 1),

            // set bit 2
            0xD0 => set_bit!(self, self.cpu.b, 2),
            0xD1 => set_bit!(self, self.cpu.c, 2),
            0xD2 => set_bit!(self, self.cpu.d, 2),
            0xD3 => set_bit!(self, self.cpu.e, 2),
            0xD4 => set_bit!(self, self.cpu.h, 2),
            0xD5 => set_bit!(self, self.cpu.l, 2),
            0xD6 => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 2),
            0xD7 => set_bit!(self, self.cpu.a, 2),

            // set bit 3
            0xD8 => set_bit!(self, self.cpu.b, 3),
            0xD9 => set_bit!(self, self.cpu.c, 3),
            0xDA => set_bit!(self, self.cpu.d, 3),
            0xDB => set_bit!(self, self.cpu.e, 3),
            0xDC => set_bit!(self, self.cpu.h, 3),
            0xDD => set_bit!(self, self.cpu.l, 3),
            0xDE => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 3),
            0xDF => set_bit!(self, self.cpu.a, 3),

            // set bit 4
            0xE0 => set_bit!(self, self.cpu.b, 4),
            0xE1 => set_bit!(self, self.cpu.c, 4),
            0xE2 => set_bit!(self, self.cpu.d, 4),
            0xE3 => set_bit!(self, self.cpu.e, 4),
            0xE4 => set_bit!(self, self.cpu.h, 4),
            0xE5 => set_bit!(self, self.cpu.l, 4),
            0xE6 => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 4),
            0xE7 => set_bit!(self, self.cpu.a, 4),

            // set bit 5
            0xE8 => set_bit!(self, self.cpu.b, 5),
            0xE9 => set_bit!(self, self.cpu.c, 5),
            0xEA => set_bit!(self, self.cpu.d, 5),
            0xEB => set_bit!(self, self.cpu.e, 5),
            0xEC => set_bit!(self, self.cpu.h, 5),
            0xED => set_bit!(self, self.cpu.l, 5),
            0xEE => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 5),
            0xEF => set_bit!(self, self.cpu.a, 5),

            // set bit 6
            0xF0 => set_bit!(self, self.cpu.b, 6),
            0xF1 => set_bit!(self, self.cpu.c, 6),
            0xF2 => set_bit!(self, self.cpu.d, 6),
            0xF3 => set_bit!(self, self.cpu.e, 6),
            0xF4 => set_bit!(self, self.cpu.h, 6),
            0xF5 => set_bit!(self, self.cpu.l, 6),
            0xF6 => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 6),
            0xF7 => set_bit!(self, self.cpu.a, 6),

            // set bit 7
            0xF8 => set_bit!(self, self.cpu.b, 7),
            0xF9 => set_bit!(self, self.cpu.c, 7),
            0xFA => set_bit!(self, self.cpu.d, 7),
            0xFB => set_bit!(self, self.cpu.e, 7),
            0xFC => set_bit!(self, self.cpu.h, 7),
            0xFD => set_bit!(self, self.cpu.l, 7),
            0xFE => set_bit_memory!(self, u16::from_bytes(self.cpu.h, self.cpu.l), 7),
            0xFF => set_bit!(self, self.cpu.a, 7),

            _ => panic!("Unknown extended opcode: {:02X}", opcode),
        }
    }

    fn jump(&mut self, flag: u8, use_condition: bool, condition: bool) -> u32 {
        let address = self.read_word();
        if !use_condition || self.cpu.f.test_bit(flag) == condition {
            self.cpu.pc = address;
        }
        12
    }

    fn jump_immediate(&mut self, flag: u8, use_condition: bool, condition: bool) -> u32 {
        let offset = self.read_byte() as i8 as i16;
        if !use_condition || self.cpu.f.test_bit(flag) == condition {
            self.cpu.pc = self.cpu.pc.wrapping_add_signed(offset);
        }
        8
    }

    fn call(&mut self, flag: u8, use_condition: bool, condition: bool) -> u32 {
        let address = self.read_word();
        if !use_condition || self.cpu.f.test_bit(flag) == condition {
            self.push_stack(self.cpu.pc);
            self.cpu.pc = address;
        }
        12
    }

    fn restart(&mut self, offset: u16) -> u32 {
        self.push_stack(self.cpu.pc);
        self.cpu.pc = offset;
        32
    }

    fn return_from_call(&mut self, flag: u8, use_condition: bool, condition: bool) -> u32 {
        if !use_condition || self.cpu.f.test_bit(flag) == condition {
            self.cpu.pc = self.pop_stack();
        }
        8
    }

    fn add_8bit(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let (result, carry) = self.cpu.a.overflowing_add(value);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu
            .f
            .toggle_bit(FLAG_HALF_CARRY, (self.cpu.a & 0x0f) + (value & 0x0f) > 0x0f);
        self.cpu.f.toggle_bit(FLAG_CARRY, carry);
        self.cpu.a = result;
        4
    }

    fn add_8bit_carry(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let carry = self.cpu.f.test_bit(FLAG_CARRY) as u8;
        let result = self.cpu.a.wrapping_add(value).wrapping_add(carry);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.toggle_bit(
            FLAG_HALF_CARRY,
            (self.cpu.a & 0x0f) + (value & 0x0f) + carry > 0x0f,
        );
        self.cpu.f.toggle_bit(
            FLAG_CARRY,
            (self.cpu.a as u16) + (value as u16) + (carry as u16) > 0xff,
        );
        self.cpu.a = result;
        4
    }

    fn sub_8bit(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let (result, carry) = self.cpu.a.overflowing_sub(value);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.set_bit(FLAG_SUBTRACT);
        self.cpu
            .f
            .toggle_bit(FLAG_HALF_CARRY, (self.cpu.a & 0x0f) < (value & 0x0f));
        self.cpu.f.toggle_bit(FLAG_CARRY, carry);
        self.cpu.a = result;
        4
    }

    fn sub_8bit_carry(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let carry = self.cpu.f.test_bit(FLAG_CARRY) as u8;
        let result = self.cpu.a.wrapping_sub(value).wrapping_sub(carry);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.set_bit(FLAG_SUBTRACT);
        self.cpu.f.toggle_bit(
            FLAG_HALF_CARRY,
            (self.cpu.a & 0x0f) < (value & 0x0f) + carry,
        );
        self.cpu.f.toggle_bit(
            FLAG_CARRY,
            (self.cpu.a as u16) < (value as u16) + (carry as u16),
        );
        self.cpu.a = result;
        4
    }

    fn and_8bit(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        self.cpu.a &= value;
        self.cpu.f.toggle_bit(FLAG_ZERO, self.cpu.a == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.set_bit(FLAG_HALF_CARRY);
        self.cpu.f.reset_bit(FLAG_CARRY);
        4
    }

    fn or_8bit(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let result = self.cpu.a | value;
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.reset_bit(FLAG_HALF_CARRY);
        self.cpu.f.reset_bit(FLAG_CARRY);
        self.cpu.a = result;
        4
    }

    fn xor_8bit(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let result = self.cpu.a ^ value;
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.reset_bit(FLAG_HALF_CARRY);
        self.cpu.f.reset_bit(FLAG_CARRY);
        self.cpu.a = result;
        4
    }

    fn compare_8bit(&mut self, value: Option<u8>) -> u32 {
        let value = value.unwrap_or_else(|| self.read_byte());
        let (result, carry) = self.cpu.a.overflowing_sub(value);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.set_bit(FLAG_SUBTRACT);
        self.cpu
            .f
            .toggle_bit(FLAG_HALF_CARRY, (self.cpu.a & 0x0f) < (value & 0x0f));
        self.cpu.f.toggle_bit(FLAG_CARRY, carry);
        4
    }

    fn inc_8bit(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.toggle_bit(FLAG_HALF_CARRY, value & 0x0f == 0x0f);
        result
    }

    fn dec_8bit(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.set_bit(FLAG_SUBTRACT);
        self.cpu.f.toggle_bit(FLAG_HALF_CARRY, value & 0x0f == 0x00);
        result
    }

    fn add_16bit(&mut self, value: u16) -> u32 {
        let hl = u16::from_bytes(self.cpu.h, self.cpu.l);
        let (result, carry) = hl.overflowing_add(value);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu
            .f
            .toggle_bit(FLAG_HALF_CARRY, hl.test_add_carry_bit(value, 11));
        self.cpu.f.toggle_bit(FLAG_CARRY, carry);
        self.cpu.h = result.hi();
        self.cpu.l = result.lo();
        8
    }
}

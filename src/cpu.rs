use std::fs::OpenOptions;

use crate::{
    constants::{
        FLAG_CARRY, FLAG_HALF_CARRY, FLAG_SUBTRACT, FLAG_ZERO, INTERRUPT_ENABLE, INTERRUPT_FLAG,
    },
    mmu::MMU,
    traits::{CarryTest, Register, SetBit, TestBit, ToggleBit},
};

pub struct CPU {
    // 8-bit registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,

    // 16-bit registers
    pub pc: u16, // program counter
    pub sp: u16, // stack pointer

    pub mmu: MMU,

    pub halted: bool,
    pub pending_interrupt: Option<bool>,
    pub interrupt_master_enable: bool,
}

/**
 * TODO:
 * - make inc_8bit take place using reference and pass a let mut value reference for hl
 * - extract some logic to apu.rs
 * - move constant to the corresponding files, e.g. SCANLINE to gpu.rs
 * - Check out the default values for io registers
 * WATCH OUT:
 * - some mmu.write_byte calls will be replaced with self.write_memory
 */

impl CPU {
    pub fn new() -> CPU {
        // https://gbdev.io/pandocs/Power_Up_Sequence.html
        CPU {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            pc: 0x100,
            sp: 0xFFFE,
            mmu: MMU::new(),
            halted: false,
            pending_interrupt: None,
            interrupt_master_enable: false,
        }
    }

    pub fn execute_next_opcode(&mut self) -> u16 {
        let cycles = if self.halted {
            4
        } else {
            let opcode = self.read_immediate_byte();
            let result = self.execute(opcode);
            result
        };

        if self.pending_interrupt == Some(true) && self.mmu.read(self.pc - 1) != 0xFB {
            self.interrupt_master_enable = true;
            self.pending_interrupt = None;
        } else if self.pending_interrupt == Some(false) && self.mmu.read(self.pc - 1) != 0xF3 {
            self.interrupt_master_enable = false;
            self.pending_interrupt = None;
        }

        cycles
    }

    pub fn do_interrupts(&mut self) {
        if self.interrupt_master_enable {
            for i in 0..5 {
                if self.mmu.interrupt_enable.test_bit(i) && self.mmu.interrupt_flag.test_bit(i) {
                    self.service_interrupt(i);
                }
            }
        }
    }

    fn service_interrupt(&mut self, id: u8) {
        self.halted = false;
        self.interrupt_master_enable = false;
        self.mmu.interrupt_flag.reset_bit(id);

        self.push_stack(self.pc);

        match id {
            0 => self.pc = 0x40,
            1 => self.pc = 0x48,
            2 => self.pc = 0x50,
            3 => self.pc = 0x58,
            4 => self.pc = 0x60,
            _ => panic!("Invalid interrupt id"),
        }
    }

    pub fn request_interrupt(&mut self, id: u8) {
        println!("Requesting interrupt {}", id);
        self.mmu.interrupt_flag.set_bit(id);
    }

    fn af(&self) -> u16 {
        u16::from_bytes(self.a, self.f)
    }

    fn bc(&self) -> u16 {
        u16::from_bytes(self.b, self.c)
    }

    fn de(&self) -> u16 {
        u16::from_bytes(self.d, self.e)
    }

    fn hl(&self) -> u16 {
        u16::from_bytes(self.h, self.l)
    }

    fn read_immediate_byte(&mut self) -> u8 {
        let result = self.mmu.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        result
    }

    fn read_immediate_word(&mut self) -> u16 {
        let lo = self.read_immediate_byte();
        let hi = self.read_immediate_byte();
        u16::from_bytes(hi, lo)
    }

    fn push_stack(&mut self, data: u16) -> u16 {
        self.mmu.write(self.sp.wrapping_sub(1), data.hi());
        self.mmu.write(self.sp.wrapping_sub(2), data.lo());
        self.sp = self.sp.wrapping_sub(2);
        16
    }

    fn pop_stack(&mut self) -> u16 {
        let lo = self.mmu.read(self.sp);
        let hi = self.mmu.read(self.sp.wrapping_add(1));
        self.sp = self.sp.wrapping_add(2);
        u16::from_bytes(hi, lo)
    }

    pub fn execute(&mut self, opcode: u8) -> u16 {
        macro_rules! load {
            ($lhs: expr, $rhs: expr) => {{
                $lhs = $rhs;
                4
            }};
        }

        macro_rules! write {
            ($addr: expr, $reg: expr) => {{
                self.mmu.write($addr, $reg);
                8
            }};
        }

        macro_rules! read {
            ($addr: expr, $reg: expr) => {{
                $reg = self.mmu.read($addr);
                8
            }};
        }

        macro_rules! load_16bit {
            ($hi: ident, $lo: ident, $word: expr) => {{
                let word = $word;
                self.$hi = word.hi();
                self.$lo = word.lo();
                12
            }};
        }

        macro_rules! op_16bit {
            ($hi: ident, $lo: ident, $op: ident) => {{
                let result = u16::from_bytes(self.$hi, self.$lo).$op(1);
                self.$hi = result.hi();
                self.$lo = result.lo();
                8
            }};
        }

        macro_rules! rotate_left {
            ($reg: expr) => {{
                let ci = self.f.test_bit(FLAG_CARRY) as u8;
                let co = $reg & 0x80;
                $reg = ($reg << 1) | ci;
                self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f.reset_bit(FLAG_HALF_CARRY);
                self.f.toggle_bit(FLAG_CARRY, co != 0);
                8
            }};
        }

        macro_rules! rotate_left_carry {
            ($reg: expr) => {{
                let co = $reg & 0x80;
                $reg = $reg.rotate_left(1);
                self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f.reset_bit(FLAG_HALF_CARRY);
                self.f.toggle_bit(FLAG_CARRY, co != 0);
                8
            }};
        }

        macro_rules! rotate_right {
            ($reg: expr) => {{
                let ci = self.f.test_bit(FLAG_CARRY) as u8;
                let co = $reg & 0x01;
                $reg = ($reg >> 1) | (ci << 7);
                self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f.reset_bit(FLAG_HALF_CARRY);
                self.f.toggle_bit(FLAG_CARRY, co != 0);
                8
            }};
        }

        macro_rules! rotate_right_carry {
            ($reg: expr) => {{
                let co = $reg & 0x01;
                $reg = $reg.rotate_right(1);
                self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f.reset_bit(FLAG_HALF_CARRY);
                self.f.toggle_bit(FLAG_CARRY, co != 0);
                8
            }};
        }

        macro_rules! without_zero {
            ($action: expr) => {{
                let cycles = $action;
                self.f.reset_bit(FLAG_ZERO);
                cycles
            }};
        }

        match opcode {
            0x00 => 4, // NOP

            // 8-bit loads
            0x06 => load!(self.b, self.read_immediate_byte()), // LD B, n
            0x0E => load!(self.c, self.read_immediate_byte()), // LD C, n
            0x16 => load!(self.d, self.read_immediate_byte()), // LD D, n
            0x1E => load!(self.e, self.read_immediate_byte()), // LD E, n
            0x26 => load!(self.h, self.read_immediate_byte()), // LD H, n
            0x2E => load!(self.l, self.read_immediate_byte()), // LD L, n

            // load register
            0x7F => load!(self.a, self.a), // LD A, A
            0x78 => load!(self.a, self.b), // LD A, B
            0x79 => load!(self.a, self.c), // LD A, C
            0x7A => load!(self.a, self.d), // LD A, D
            0x7B => load!(self.a, self.e), // LD A, E
            0x7C => load!(self.a, self.h), // LD A, H
            0x7D => load!(self.a, self.l), // LD A, L

            0x47 => load!(self.b, self.a), // LD B, A
            0x40 => load!(self.b, self.b), // LD B, B
            0x41 => load!(self.b, self.c), // LD B, C
            0x42 => load!(self.b, self.d), // LD B, D
            0x43 => load!(self.b, self.e), // LD B, E
            0x44 => load!(self.b, self.h), // LD B, H
            0x45 => load!(self.b, self.l), // LD B, L

            0x4F => load!(self.c, self.a), // LD C, A
            0x48 => load!(self.c, self.b), // LD C, B
            0x49 => load!(self.c, self.c), // LD C, C
            0x4A => load!(self.c, self.d), // LD C, D
            0x4B => load!(self.c, self.e), // LD C, E
            0x4C => load!(self.c, self.h), // LD C, H
            0x4D => load!(self.c, self.l), // LD C, L

            0x57 => load!(self.d, self.a), // LD D, A
            0x50 => load!(self.d, self.b), // LD D, B
            0x51 => load!(self.d, self.c), // LD D, C
            0x52 => load!(self.d, self.d), // LD D, D
            0x53 => load!(self.d, self.e), // LD D, E
            0x54 => load!(self.d, self.h), // LD D, H
            0x55 => load!(self.d, self.l), // LD D, L

            0x5F => load!(self.e, self.a), // LD E, A
            0x58 => load!(self.e, self.b), // LD E, B
            0x59 => load!(self.e, self.c), // LD E, C
            0x5A => load!(self.e, self.d), // LD E, D
            0x5B => load!(self.e, self.e), // LD E, E
            0x5C => load!(self.e, self.h), // LD E, H
            0x5D => load!(self.e, self.l), // LD E, L

            0x67 => load!(self.h, self.a), // LD H, A
            0x60 => load!(self.h, self.b), // LD H, B
            0x61 => load!(self.h, self.c), // LD H, C
            0x62 => load!(self.h, self.d), // LD H, D
            0x63 => load!(self.h, self.e), // LD H, E
            0x64 => load!(self.h, self.h), // LD H, H
            0x65 => load!(self.h, self.l), // LD H, L

            0x6F => load!(self.l, self.a), // LD L, A
            0x68 => load!(self.l, self.b), // LD L, B
            0x69 => load!(self.l, self.c), // LD L, C
            0x6A => load!(self.l, self.d), // LD L, D
            0x6B => load!(self.l, self.e), // LD L, E
            0x6C => load!(self.l, self.h), // LD L, H
            0x6D => load!(self.l, self.l), // LD L, L

            // write register to memory
            0x70 => write!(self.hl(), self.b),
            0x71 => write!(self.hl(), self.c),
            0x72 => write!(self.hl(), self.d),
            0x73 => write!(self.hl(), self.e),
            0x74 => write!(self.hl(), self.h),
            0x75 => write!(self.hl(), self.l),

            // write memory to register
            0x7E => read!(self.hl(), self.a),
            0x46 => read!(self.hl(), self.b),
            0x4E => read!(self.hl(), self.c),
            0x56 => read!(self.hl(), self.d),
            0x5E => read!(self.hl(), self.e),
            0x66 => read!(self.hl(), self.h),
            0x6E => read!(self.hl(), self.l),
            0x0A => read!(self.bc(), self.a),
            0x1A => read!(self.de(), self.a),
            0xF2 => read!(u16::from_bytes(0xFF, self.c), self.a),

            // put a into memory address
            0x02 => write!(self.bc(), self.a),
            0x12 => write!(self.de(), self.a),
            0x77 => write!(self.hl(), self.a),
            0xE2 => write!(u16::from_bytes(0xFF, self.c), self.a),

            // put memory into a, decrement/increment HL
            0x3A => read!(self.hl(), self.a) + op_16bit!(h, l, wrapping_sub),
            0x2A => read!(self.hl(), self.a) + op_16bit!(h, l, wrapping_add),

            // put a into memory, decrement/increment memory
            0x32 => write!(self.hl(), self.a) + op_16bit!(h, l, wrapping_sub),
            0x22 => write!(self.hl(), self.a) + op_16bit!(h, l, wrapping_add),

            // 16 bit loads
            0x01 => load_16bit!(b, c, self.read_immediate_word()),
            0x11 => load_16bit!(d, e, self.read_immediate_word()),
            0x21 => load_16bit!(h, l, self.read_immediate_word()),

            0x31 => {
                self.sp = self.read_immediate_word();
                12
            }
            0xF9 => {
                self.sp = self.hl();
                8
            }

            // push word onto stack
            0xF5 => self.push_stack(self.af()),
            0xC5 => self.push_stack(self.bc()),
            0xD5 => self.push_stack(self.de()),
            0xE5 => self.push_stack(self.hl()),

            // // pop word from stack into register
            0xF1 => {
                let word = self.pop_stack();
                self.a = word.hi();
                self.f = word.lo() & 0xF0;
                12
            }
            0xC1 => load_16bit!(b, c, self.pop_stack()),
            0xD1 => load_16bit!(d, e, self.pop_stack()),
            0xE1 => load_16bit!(h, l, self.pop_stack()),

            // 8-bit add
            0x87 => self.add_8bit(Some(self.a)),
            0x80 => self.add_8bit(Some(self.b)),
            0x81 => self.add_8bit(Some(self.c)),
            0x82 => self.add_8bit(Some(self.d)),
            0x83 => self.add_8bit(Some(self.e)),
            0x84 => self.add_8bit(Some(self.h)),
            0x85 => self.add_8bit(Some(self.l)),
            0x86 => self.add_8bit(Some(self.mmu.read(self.hl()))) + 4,
            0xC6 => self.add_8bit(None) + 4,

            // 8-bit add + carry
            0x8F => self.add_8bit_carry(Some(self.a)),
            0x88 => self.add_8bit_carry(Some(self.b)),
            0x89 => self.add_8bit_carry(Some(self.c)),
            0x8A => self.add_8bit_carry(Some(self.d)),
            0x8B => self.add_8bit_carry(Some(self.e)),
            0x8C => self.add_8bit_carry(Some(self.h)),
            0x8D => self.add_8bit_carry(Some(self.l)),
            0x8E => self.add_8bit_carry(Some(self.mmu.read(self.hl()))) + 4,
            0xCE => self.add_8bit_carry(None) + 4,

            // 8-bit subtract
            0x97 => self.sub_8bit(Some(self.a)),
            0x90 => self.sub_8bit(Some(self.b)),
            0x91 => self.sub_8bit(Some(self.c)),
            0x92 => self.sub_8bit(Some(self.d)),
            0x93 => self.sub_8bit(Some(self.e)),
            0x94 => self.sub_8bit(Some(self.h)),
            0x95 => self.sub_8bit(Some(self.l)),
            0x96 => self.sub_8bit(Some(self.mmu.read(self.hl()))) + 4,
            0xD6 => self.sub_8bit(None) + 4,

            // 8-bit subtract + carry
            0x9F => self.sub_8bit_carry(Some(self.a)),
            0x98 => self.sub_8bit_carry(Some(self.b)),
            0x99 => self.sub_8bit_carry(Some(self.c)),
            0x9A => self.sub_8bit_carry(Some(self.d)),
            0x9B => self.sub_8bit_carry(Some(self.e)),
            0x9C => self.sub_8bit_carry(Some(self.h)),
            0x9D => self.sub_8bit_carry(Some(self.l)),
            0x9E => self.sub_8bit_carry(Some(self.mmu.read(self.hl()))) + 4,
            0xDE => self.sub_8bit_carry(None) + 4,

            // 8-bit AND
            0xA7 => self.and_8bit(Some(self.a)),
            0xA0 => self.and_8bit(Some(self.b)),
            0xA1 => self.and_8bit(Some(self.c)),
            0xA2 => self.and_8bit(Some(self.d)),
            0xA3 => self.and_8bit(Some(self.e)),
            0xA4 => self.and_8bit(Some(self.h)),
            0xA5 => self.and_8bit(Some(self.l)),
            0xA6 => self.and_8bit(Some(self.mmu.read(self.hl()))) + 4,
            0xE6 => self.and_8bit(None) + 4,

            // 8-bit OR
            0xB7 => self.or_8bit(Some(self.a)),
            0xB0 => self.or_8bit(Some(self.b)),
            0xB1 => self.or_8bit(Some(self.c)),
            0xB2 => self.or_8bit(Some(self.d)),
            0xB3 => self.or_8bit(Some(self.e)),
            0xB4 => self.or_8bit(Some(self.h)),
            0xB5 => self.or_8bit(Some(self.l)),
            0xB6 => self.or_8bit(Some(self.mmu.read(self.hl()))) + 4,
            0xF6 => self.or_8bit(None) + 4,

            // 8-bit XOR
            0xAF => self.xor_8bit(Some(self.a)),
            0xA8 => self.xor_8bit(Some(self.b)),
            0xA9 => self.xor_8bit(Some(self.c)),
            0xAA => self.xor_8bit(Some(self.d)),
            0xAB => self.xor_8bit(Some(self.e)),
            0xAC => self.xor_8bit(Some(self.h)),
            0xAD => self.xor_8bit(Some(self.l)),
            0xAE => self.xor_8bit(Some(self.mmu.read(self.hl()))) + 4,
            0xEE => self.xor_8bit(None) + 4,

            // 8-bit compare
            0xBF => self.compare_8bit(Some(self.a)),
            0xB8 => self.compare_8bit(Some(self.b)),
            0xB9 => self.compare_8bit(Some(self.c)),
            0xBA => self.compare_8bit(Some(self.d)),
            0xBB => self.compare_8bit(Some(self.e)),
            0xBC => self.compare_8bit(Some(self.h)),
            0xBD => self.compare_8bit(Some(self.l)),
            0xBE => self.compare_8bit(Some(self.mmu.read(self.hl()))) + 4,
            0xFE => self.compare_8bit(None) + 4,

            // 8-bit increment
            0x3C => {
                self.a = self.inc_8bit(self.a);
                4
            }
            0x04 => {
                self.b = self.inc_8bit(self.b);
                4
            }
            0x0C => {
                self.c = self.inc_8bit(self.c);
                4
            }
            0x14 => {
                self.d = self.inc_8bit(self.d);
                4
            }
            0x1C => {
                self.e = self.inc_8bit(self.e);
                4
            }
            0x24 => {
                self.h = self.inc_8bit(self.h);
                4
            }
            0x2C => {
                self.l = self.inc_8bit(self.l);
                4
            }
            0x34 => {
                let hl = self.hl();
                let value = self.inc_8bit(self.mmu.read(hl));
                self.mmu.write(hl, value);
                12
            }

            // 8-bit decrement
            0x3D => {
                self.a = self.dec_8bit(self.a);
                4
            }
            0x05 => {
                self.b = self.dec_8bit(self.b);
                4
            }
            0x0D => {
                self.c = self.dec_8bit(self.c);
                4
            }
            0x15 => {
                self.d = self.dec_8bit(self.d);
                4
            }
            0x1D => {
                self.e = self.dec_8bit(self.e);
                4
            }

            0x25 => {
                self.h = self.dec_8bit(self.h);
                4
            }
            0x2D => {
                self.l = self.dec_8bit(self.l);
                4
            }
            0x35 => {
                let hl = self.hl();
                let value = self.dec_8bit(self.mmu.read(hl));
                self.mmu.write(hl, value);
                12
            }

            // 16-bit add
            0x09 => self.add_16bit(u16::from_bytes(self.b, self.c)),
            0x19 => self.add_16bit(u16::from_bytes(self.d, self.e)),
            0x29 => self.add_16bit(u16::from_bytes(self.h, self.l)),
            0x39 => self.add_16bit(self.sp),

            // 16-bit increment
            0x03 => op_16bit!(b, c, wrapping_add),
            0x13 => op_16bit!(d, e, wrapping_add),
            0x23 => op_16bit!(h, l, wrapping_add),
            0x33 => {
                self.sp = self.sp.wrapping_add(1);
                8
            }

            // 16-bit decrement
            0x0B => op_16bit!(b, c, wrapping_sub),
            0x1B => op_16bit!(d, e, wrapping_sub),
            0x2B => op_16bit!(h, l, wrapping_sub),
            0x3B => {
                self.sp = self.sp.wrapping_sub(1);
                8
            }

            // jumps
            0xE9 => {
                self.pc = u16::from_bytes(self.h, self.l);
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
                if !self.f.test_bit(FLAG_SUBTRACT) {
                    if self.f.test_bit(FLAG_CARRY) || self.a > 0x99 {
                        self.a = self.a.wrapping_add(0x60);
                        carry = true;
                    }
                    if self.f.test_bit(FLAG_HALF_CARRY) || self.a & 0x0f > 0x09 {
                        self.a = self.a.wrapping_add(0x06);
                    }
                } else if self.f.test_bit(FLAG_CARRY) {
                    carry = true;
                    let adder = if self.f.test_bit(FLAG_HALF_CARRY) {
                        0x9a
                    } else {
                        0xa0
                    };
                    self.a = self.a.wrapping_add(adder);
                } else if self.f.test_bit(FLAG_HALF_CARRY) {
                    self.a = self.a.wrapping_add(0xfa);
                }
                self.f.toggle_bit(FLAG_ZERO, self.a == 0);
                self.f.reset_bit(FLAG_HALF_CARRY);
                self.f.toggle_bit(FLAG_CARRY, carry);
                4
            }

            0x07 => without_zero!(rotate_left_carry!(self.a)),
            0x0F => without_zero!(rotate_right_carry!(self.a)),
            0x17 => without_zero!(rotate_left!(self.a)),
            0x1F => without_zero!(rotate_right!(self.a)),

            0xD9 => {
                self.pc = self.pop_stack();
                self.interrupt_master_enable = true;
                8
            }

            0x08 => {
                let address = self.read_immediate_word();
                self.mmu.write(address, self.sp.lo());
                self.mmu.write(address.wrapping_add(1), self.sp.hi());
                20
            }

            0x36 => {
                let byte = self.read_immediate_byte();
                self.mmu.write(self.hl(), byte);
                12
            }

            0xFA => {
                let address = self.read_immediate_word();
                self.a = self.mmu.read(address);
                16
            }

            0x3E => {
                self.a = self.read_immediate_byte();
                8
            }

            0xEA => {
                let address = self.read_immediate_word();
                self.mmu.write(address, self.a);
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
                let address = u16::from_bytes(0xFF, self.read_immediate_byte());
                self.mmu.write(address, self.a);
                12
            }

            0xF0 => {
                let address = u16::from_bytes(0xFF, self.read_immediate_byte());
                self.a = self.mmu.read(address);
                12
            }

            0x2F => {
                self.a ^= 0xFF;
                self.f.set_bit(FLAG_SUBTRACT);
                self.f.set_bit(FLAG_HALF_CARRY);
                4
            }

            0x76 => {
                self.halted = true;
                4
            }

            0x3F => {
                self.f.toggle_bit(FLAG_CARRY, !self.f.test_bit(FLAG_CARRY));
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f.reset_bit(FLAG_HALF_CARRY);
                4
            }

            0x37 => {
                self.f.set_bit(FLAG_CARRY);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f.reset_bit(FLAG_HALF_CARRY);
                4
            }

            0xF8 => {
                let offset = self.read_immediate_byte() as i8 as i16 as u16;
                let result = self.sp.wrapping_add(offset);
                self.f.reset_bit(FLAG_ZERO);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f
                    .toggle_bit(FLAG_HALF_CARRY, self.sp.test_add_carry_bit(offset, 3));
                self.f
                    .toggle_bit(FLAG_CARRY, self.sp.test_add_carry_bit(offset, 7));
                self.h = result.hi();
                self.l = result.lo();
                12
            }

            0x10 => {
                self.pc += 1;
                4
            }

            0xe8 => {
                let offset = self.read_immediate_byte() as i8 as i16 as u16;
                let result = self.sp.wrapping_add(offset);
                self.f.reset_bit(FLAG_ZERO);
                self.f.reset_bit(FLAG_SUBTRACT);
                self.f
                    .toggle_bit(FLAG_HALF_CARRY, self.sp.test_add_carry_bit(offset, 3));
                self.f
                    .toggle_bit(FLAG_CARRY, self.sp.test_add_carry_bit(offset, 7));
                self.sp = result;
                16
            }

            0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => {
                panic!("Unknown opcode: {:02X}", opcode)
            }

            0xCB => {
                let extended_opcode = self.read_immediate_byte();

                macro_rules! perform_in_memory {
                    ($macro: ident $(, $arg: expr)*) => {{
                        let hl = self.hl();
                        let mut reg = self.mmu.read(hl);
                        let cycles = $macro!(reg $(, $arg)*);
                        self.mmu.write(hl, reg);
                        cycles + 8
                    }};
                }

                macro_rules! shift_left_arithmetic {
                    ($reg: expr) => {{
                        let is_msb_set = $reg.test_bit(7);
                        $reg <<= 1;
                        self.f.toggle_bit(FLAG_CARRY, is_msb_set);
                        self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                        self.f.reset_bit(FLAG_SUBTRACT);
                        self.f.reset_bit(FLAG_HALF_CARRY);
                        8
                    }};
                }

                macro_rules! shift_right_arithmetic {
                    ($reg: expr) => {{
                        let is_lsb_set = $reg.test_bit(0);
                        let is_msb_set = $reg.test_bit(7);
                        $reg >>= 1;
                        $reg.toggle_bit(7, is_msb_set);
                        self.f.toggle_bit(FLAG_CARRY, is_lsb_set);
                        self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                        self.f.reset_bit(FLAG_SUBTRACT);
                        self.f.reset_bit(FLAG_HALF_CARRY);
                        8
                    }};
                }

                macro_rules! shift_right_logical {
                    ($reg: expr) => {{
                        let is_lsb_set = $reg.test_bit(0);
                        $reg >>= 1;
                        self.f.toggle_bit(FLAG_CARRY, is_lsb_set);
                        self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                        self.f.reset_bit(FLAG_SUBTRACT);
                        self.f.reset_bit(FLAG_HALF_CARRY);
                        8
                    }};
                }

                macro_rules! swap_nibbles {
                    ($reg: expr) => {{
                        $reg = ($reg << 4) | ($reg >> 4);
                        self.f.toggle_bit(FLAG_ZERO, $reg == 0);
                        self.f.reset_bit(FLAG_SUBTRACT);
                        self.f.reset_bit(FLAG_HALF_CARRY);
                        self.f.reset_bit(FLAG_CARRY);
                        8
                    }};
                }

                macro_rules! test_bit {
                    ($reg: expr, $bit: expr) => {{
                        self.f.toggle_bit(FLAG_ZERO, !$reg.test_bit($bit));
                        self.f.reset_bit(FLAG_SUBTRACT);
                        self.f.set_bit(FLAG_HALF_CARRY);
                        8
                    }};
                }

                macro_rules! reset_bit {
                    ($reg: expr, $bit: expr) => {{
                        $reg.reset_bit($bit);
                        8
                    }};
                }

                macro_rules! set_bit {
                    ($reg: expr, $bit: expr) => {{
                        $reg.set_bit($bit);
                        8
                    }};
                }

                match extended_opcode {
                    // rotate left carry
                    0x07 => rotate_left_carry!(self.a),
                    0x00 => rotate_left_carry!(self.b),
                    0x01 => rotate_left_carry!(self.c),
                    0x02 => rotate_left_carry!(self.d),
                    0x03 => rotate_left_carry!(self.e),
                    0x04 => rotate_left_carry!(self.h),
                    0x05 => rotate_left_carry!(self.l),
                    0x06 => perform_in_memory!(rotate_left_carry),

                    // rotate right carry
                    0x0F => rotate_right_carry!(self.a),
                    0x08 => rotate_right_carry!(self.b),
                    0x09 => rotate_right_carry!(self.c),
                    0x0A => rotate_right_carry!(self.d),
                    0x0B => rotate_right_carry!(self.e),
                    0x0C => rotate_right_carry!(self.h),
                    0x0D => rotate_right_carry!(self.l),
                    0x0E => perform_in_memory!(rotate_right_carry),

                    // rotate left
                    0x17 => rotate_left!(self.a),
                    0x10 => rotate_left!(self.b),
                    0x11 => rotate_left!(self.c),
                    0x12 => rotate_left!(self.d),
                    0x13 => rotate_left!(self.e),
                    0x14 => rotate_left!(self.h),
                    0x15 => rotate_left!(self.l),
                    0x16 => perform_in_memory!(rotate_left),

                    // rotate right
                    0x1F => rotate_right!(self.a),
                    0x18 => rotate_right!(self.b),
                    0x19 => rotate_right!(self.c),
                    0x1A => rotate_right!(self.d),
                    0x1B => rotate_right!(self.e),
                    0x1C => rotate_right!(self.h),
                    0x1D => rotate_right!(self.l),
                    0x1E => perform_in_memory!(rotate_right),

                    // shift left arithmetic
                    0x27 => shift_left_arithmetic!(self.a),
                    0x20 => shift_left_arithmetic!(self.b),
                    0x21 => shift_left_arithmetic!(self.c),
                    0x22 => shift_left_arithmetic!(self.d),
                    0x23 => shift_left_arithmetic!(self.e),
                    0x24 => shift_left_arithmetic!(self.h),
                    0x25 => shift_left_arithmetic!(self.l),
                    0x26 => perform_in_memory!(shift_left_arithmetic),

                    // shift right arithmetic
                    0x2F => shift_right_arithmetic!(self.a),
                    0x28 => shift_right_arithmetic!(self.b),
                    0x29 => shift_right_arithmetic!(self.c),
                    0x2A => shift_right_arithmetic!(self.d),
                    0x2B => shift_right_arithmetic!(self.e),
                    0x2C => shift_right_arithmetic!(self.h),
                    0x2D => shift_right_arithmetic!(self.l),
                    0x2E => perform_in_memory!(shift_right_arithmetic),

                    // shift right logical
                    0x3F => shift_right_logical!(self.a),
                    0x38 => shift_right_logical!(self.b),
                    0x39 => shift_right_logical!(self.c),
                    0x3A => shift_right_logical!(self.d),
                    0x3B => shift_right_logical!(self.e),
                    0x3C => shift_right_logical!(self.h),
                    0x3D => shift_right_logical!(self.l),
                    0x3E => perform_in_memory!(shift_right_logical),

                    // swap nibbles
                    0x37 => swap_nibbles!(self.a),
                    0x30 => swap_nibbles!(self.b),
                    0x31 => swap_nibbles!(self.c),
                    0x32 => swap_nibbles!(self.d),
                    0x33 => swap_nibbles!(self.e),
                    0x34 => swap_nibbles!(self.h),
                    0x35 => swap_nibbles!(self.l),
                    0x36 => perform_in_memory!(swap_nibbles),

                    // test bit 0
                    0x47 => test_bit!(self.a, 0),
                    0x40 => test_bit!(self.b, 0),
                    0x41 => test_bit!(self.c, 0),
                    0x42 => test_bit!(self.d, 0),
                    0x43 => test_bit!(self.e, 0),
                    0x44 => test_bit!(self.h, 0),
                    0x45 => test_bit!(self.l, 0),
                    0x46 => perform_in_memory!(test_bit, 0),

                    // test bit 1
                    0x4F => test_bit!(self.a, 1),
                    0x48 => test_bit!(self.b, 1),
                    0x49 => test_bit!(self.c, 1),
                    0x4A => test_bit!(self.d, 1),
                    0x4B => test_bit!(self.e, 1),
                    0x4C => test_bit!(self.h, 1),
                    0x4D => test_bit!(self.l, 1),
                    0x4E => perform_in_memory!(test_bit, 1),

                    // test bit 2
                    0x57 => test_bit!(self.a, 2),
                    0x50 => test_bit!(self.b, 2),
                    0x51 => test_bit!(self.c, 2),
                    0x52 => test_bit!(self.d, 2),
                    0x53 => test_bit!(self.e, 2),
                    0x54 => test_bit!(self.h, 2),
                    0x55 => test_bit!(self.l, 2),
                    0x56 => perform_in_memory!(test_bit, 2),

                    // test bit 3
                    0x5F => test_bit!(self.a, 3),
                    0x58 => test_bit!(self.b, 3),
                    0x59 => test_bit!(self.c, 3),
                    0x5A => test_bit!(self.d, 3),
                    0x5B => test_bit!(self.e, 3),
                    0x5C => test_bit!(self.h, 3),
                    0x5D => test_bit!(self.l, 3),
                    0x5E => perform_in_memory!(test_bit, 3),

                    // test bit 4
                    0x67 => test_bit!(self.a, 4),
                    0x60 => test_bit!(self.b, 4),
                    0x61 => test_bit!(self.c, 4),
                    0x62 => test_bit!(self.d, 4),
                    0x63 => test_bit!(self.e, 4),
                    0x64 => test_bit!(self.h, 4),
                    0x65 => test_bit!(self.l, 4),
                    0x66 => perform_in_memory!(test_bit, 4),

                    // test bit 5
                    0x6F => test_bit!(self.a, 5),
                    0x68 => test_bit!(self.b, 5),
                    0x69 => test_bit!(self.c, 5),
                    0x6A => test_bit!(self.d, 5),
                    0x6B => test_bit!(self.e, 5),
                    0x6C => test_bit!(self.h, 5),
                    0x6D => test_bit!(self.l, 5),
                    0x6E => perform_in_memory!(test_bit, 5),

                    // test bit 6
                    0x77 => test_bit!(self.a, 6),
                    0x70 => test_bit!(self.b, 6),
                    0x71 => test_bit!(self.c, 6),
                    0x72 => test_bit!(self.d, 6),
                    0x73 => test_bit!(self.e, 6),
                    0x74 => test_bit!(self.h, 6),
                    0x75 => test_bit!(self.l, 6),
                    0x76 => perform_in_memory!(test_bit, 6),

                    // test bit 7
                    0x7F => test_bit!(self.a, 7),
                    0x78 => test_bit!(self.b, 7),
                    0x79 => test_bit!(self.c, 7),
                    0x7A => test_bit!(self.d, 7),
                    0x7B => test_bit!(self.e, 7),
                    0x7C => test_bit!(self.h, 7),
                    0x7D => test_bit!(self.l, 7),
                    0x7E => perform_in_memory!(test_bit, 7),

                    // reset bit 0
                    0x87 => reset_bit!(self.a, 0),
                    0x80 => reset_bit!(self.b, 0),
                    0x81 => reset_bit!(self.c, 0),
                    0x82 => reset_bit!(self.d, 0),
                    0x83 => reset_bit!(self.e, 0),
                    0x84 => reset_bit!(self.h, 0),
                    0x85 => reset_bit!(self.l, 0),
                    0x86 => perform_in_memory!(reset_bit, 0),

                    // reset bit 1
                    0x8F => reset_bit!(self.a, 1),
                    0x88 => reset_bit!(self.b, 1),
                    0x89 => reset_bit!(self.c, 1),
                    0x8A => reset_bit!(self.d, 1),
                    0x8B => reset_bit!(self.e, 1),
                    0x8C => reset_bit!(self.h, 1),
                    0x8D => reset_bit!(self.l, 1),
                    0x8E => perform_in_memory!(reset_bit, 1),

                    // reset bit 2
                    0x97 => reset_bit!(self.a, 2),
                    0x90 => reset_bit!(self.b, 2),
                    0x91 => reset_bit!(self.c, 2),
                    0x92 => reset_bit!(self.d, 2),
                    0x93 => reset_bit!(self.e, 2),
                    0x94 => reset_bit!(self.h, 2),
                    0x95 => reset_bit!(self.l, 2),
                    0x96 => perform_in_memory!(reset_bit, 2),

                    // reset bit 3
                    0x9F => reset_bit!(self.a, 3),
                    0x98 => reset_bit!(self.b, 3),
                    0x99 => reset_bit!(self.c, 3),
                    0x9A => reset_bit!(self.d, 3),
                    0x9B => reset_bit!(self.e, 3),
                    0x9C => reset_bit!(self.h, 3),
                    0x9D => reset_bit!(self.l, 3),
                    0x9E => perform_in_memory!(reset_bit, 3),

                    // reset bit 4
                    0xA7 => reset_bit!(self.a, 4),
                    0xA0 => reset_bit!(self.b, 4),
                    0xA1 => reset_bit!(self.c, 4),
                    0xA2 => reset_bit!(self.d, 4),
                    0xA3 => reset_bit!(self.e, 4),
                    0xA4 => reset_bit!(self.h, 4),
                    0xA5 => reset_bit!(self.l, 4),
                    0xA6 => perform_in_memory!(reset_bit, 4),

                    // reset bit 5
                    0xAF => reset_bit!(self.a, 5),
                    0xA8 => reset_bit!(self.b, 5),
                    0xA9 => reset_bit!(self.c, 5),
                    0xAA => reset_bit!(self.d, 5),
                    0xAB => reset_bit!(self.e, 5),
                    0xAC => reset_bit!(self.h, 5),
                    0xAD => reset_bit!(self.l, 5),
                    0xAE => perform_in_memory!(reset_bit, 5),

                    // reset bit 6
                    0xB7 => reset_bit!(self.a, 6),
                    0xB0 => reset_bit!(self.b, 6),
                    0xB1 => reset_bit!(self.c, 6),
                    0xB2 => reset_bit!(self.d, 6),
                    0xB3 => reset_bit!(self.e, 6),
                    0xB4 => reset_bit!(self.h, 6),
                    0xB5 => reset_bit!(self.l, 6),
                    0xB6 => perform_in_memory!(reset_bit, 6),

                    // reset bit 7
                    0xBF => reset_bit!(self.a, 7),
                    0xB8 => reset_bit!(self.b, 7),
                    0xB9 => reset_bit!(self.c, 7),
                    0xBA => reset_bit!(self.d, 7),
                    0xBB => reset_bit!(self.e, 7),
                    0xBC => reset_bit!(self.h, 7),
                    0xBD => reset_bit!(self.l, 7),
                    0xBE => perform_in_memory!(reset_bit, 7),

                    // set bit 0
                    0xC7 => set_bit!(self.a, 0),
                    0xC0 => set_bit!(self.b, 0),
                    0xC1 => set_bit!(self.c, 0),
                    0xC2 => set_bit!(self.d, 0),
                    0xC3 => set_bit!(self.e, 0),
                    0xC4 => set_bit!(self.h, 0),
                    0xC5 => set_bit!(self.l, 0),
                    0xC6 => perform_in_memory!(set_bit, 0),

                    // set bit 1
                    0xCF => set_bit!(self.a, 1),
                    0xC8 => set_bit!(self.b, 1),
                    0xC9 => set_bit!(self.c, 1),
                    0xCA => set_bit!(self.d, 1),
                    0xCB => set_bit!(self.e, 1),
                    0xCC => set_bit!(self.h, 1),
                    0xCD => set_bit!(self.l, 1),
                    0xCE => perform_in_memory!(set_bit, 1),

                    // set bit 2
                    0xD7 => set_bit!(self.a, 2),
                    0xD0 => set_bit!(self.b, 2),
                    0xD1 => set_bit!(self.c, 2),
                    0xD2 => set_bit!(self.d, 2),
                    0xD3 => set_bit!(self.e, 2),
                    0xD4 => set_bit!(self.h, 2),
                    0xD5 => set_bit!(self.l, 2),
                    0xD6 => perform_in_memory!(set_bit, 2),

                    // set bit 3
                    0xDF => set_bit!(self.a, 3),
                    0xD8 => set_bit!(self.b, 3),
                    0xD9 => set_bit!(self.c, 3),
                    0xDA => set_bit!(self.d, 3),
                    0xDB => set_bit!(self.e, 3),
                    0xDC => set_bit!(self.h, 3),
                    0xDD => set_bit!(self.l, 3),
                    0xDE => perform_in_memory!(set_bit, 3),

                    // set bit 4
                    0xE7 => set_bit!(self.a, 4),
                    0xE0 => set_bit!(self.b, 4),
                    0xE1 => set_bit!(self.c, 4),
                    0xE2 => set_bit!(self.d, 4),
                    0xE3 => set_bit!(self.e, 4),
                    0xE4 => set_bit!(self.h, 4),
                    0xE5 => set_bit!(self.l, 4),
                    0xE6 => perform_in_memory!(set_bit, 4),

                    // set bit 5
                    0xEF => set_bit!(self.a, 5),
                    0xE8 => set_bit!(self.b, 5),
                    0xE9 => set_bit!(self.c, 5),
                    0xEA => set_bit!(self.d, 5),
                    0xEB => set_bit!(self.e, 5),
                    0xEC => set_bit!(self.h, 5),
                    0xED => set_bit!(self.l, 5),
                    0xEE => perform_in_memory!(set_bit, 5),

                    // set bit 6
                    0xF7 => set_bit!(self.a, 6),
                    0xF0 => set_bit!(self.b, 6),
                    0xF1 => set_bit!(self.c, 6),
                    0xF2 => set_bit!(self.d, 6),
                    0xF3 => set_bit!(self.e, 6),
                    0xF4 => set_bit!(self.h, 6),
                    0xF5 => set_bit!(self.l, 6),
                    0xF6 => perform_in_memory!(set_bit, 6),

                    // set bit 7
                    0xFF => set_bit!(self.a, 7),
                    0xF8 => set_bit!(self.b, 7),
                    0xF9 => set_bit!(self.c, 7),
                    0xFA => set_bit!(self.d, 7),
                    0xFB => set_bit!(self.e, 7),
                    0xFC => set_bit!(self.h, 7),
                    0xFD => set_bit!(self.l, 7),
                    0xFE => perform_in_memory!(set_bit, 7),
                }
            }
        }
    }

    fn jump(&mut self, flag: u8, use_condition: bool, condition: bool) -> u16 {
        let address = self.read_immediate_word();
        if !use_condition || self.f.test_bit(flag) == condition {
            self.pc = address;
            return 16;
        }
        12
    }

    fn jump_immediate(&mut self, flag: u8, use_condition: bool, condition: bool) -> u16 {
        let offset = self.read_immediate_byte() as i8 as i16;
        if !use_condition || self.f.test_bit(flag) == condition {
            self.pc = self.pc.wrapping_add_signed(offset);
            return 12;
        }
        8
    }

    fn call(&mut self, flag: u8, use_condition: bool, condition: bool) -> u16 {
        let address = self.read_immediate_word();
        if !use_condition || self.f.test_bit(flag) == condition {
            self.push_stack(self.pc);
            self.pc = address;
            return 24;
        }
        12
    }

    fn restart(&mut self, offset: u16) -> u16 {
        self.push_stack(self.pc);
        self.pc = offset;
        32
    }

    fn return_from_call(&mut self, flag: u8, use_condition: bool, condition: bool) -> u16 {
        if !use_condition || self.f.test_bit(flag) == condition {
            self.pc = self.pop_stack();
            return 20;
        }
        8
    }

    fn add_16bit(&mut self, value: u16) -> u16 {
        let hl = u16::from_bytes(self.h, self.l);
        let (result, carry) = hl.overflowing_add(value);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, hl.test_add_carry_bit(value, 11));
        self.f.toggle_bit(FLAG_CARRY, carry);
        self.h = result.hi();
        self.l = result.lo();
        8
    }

    fn add_8bit(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let (result, carry) = self.a.overflowing_add(value);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, (self.a & 0x0f) + (value & 0x0f) > 0x0f);
        self.f.toggle_bit(FLAG_CARRY, carry);
        self.a = result;
        4
    }

    fn add_8bit_carry(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let carry = self.f.test_bit(FLAG_CARRY) as u8;
        let result = self.a.wrapping_add(value).wrapping_add(carry);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.toggle_bit(
            FLAG_HALF_CARRY,
            (self.a & 0x0f) + (value & 0x0f) + carry > 0x0f,
        );
        self.f.toggle_bit(
            FLAG_CARRY,
            (self.a as u16) + (value as u16) + (carry as u16) > 0xff,
        );
        self.a = result;
        4
    }

    fn sub_8bit(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let (result, carry) = self.a.overflowing_sub(value);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.set_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, (self.a & 0x0f) < (value & 0x0f));
        self.f.toggle_bit(FLAG_CARRY, carry);
        self.a = result;
        4
    }

    fn sub_8bit_carry(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let carry = self.f.test_bit(FLAG_CARRY) as u8;
        let result = self.a.wrapping_sub(value).wrapping_sub(carry);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.set_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, (self.a & 0x0f) < (value & 0x0f) + carry);
        self.f.toggle_bit(
            FLAG_CARRY,
            (self.a as u16) < (value as u16) + (carry as u16),
        );
        self.a = result;
        4
    }

    fn and_8bit(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        self.a &= value;
        self.f.toggle_bit(FLAG_ZERO, self.a == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.set_bit(FLAG_HALF_CARRY);
        self.f.reset_bit(FLAG_CARRY);
        4
    }

    fn or_8bit(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let result = self.a | value;
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.reset_bit(FLAG_HALF_CARRY);
        self.f.reset_bit(FLAG_CARRY);
        self.a = result;
        4
    }

    fn xor_8bit(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let result = self.a ^ value;
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.reset_bit(FLAG_HALF_CARRY);
        self.f.reset_bit(FLAG_CARRY);
        self.a = result;
        4
    }

    fn compare_8bit(&mut self, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| self.read_immediate_byte());
        let (result, carry) = self.a.overflowing_sub(value);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.set_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, (self.a & 0x0f) < (value & 0x0f));
        self.f.toggle_bit(FLAG_CARRY, carry);
        4
    }

    fn inc_8bit(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.toggle_bit(FLAG_HALF_CARRY, value & 0x0f == 0x0f);
        result
    }

    fn dec_8bit(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.set_bit(FLAG_SUBTRACT);
        self.f.toggle_bit(FLAG_HALF_CARRY, value & 0x0f == 0x00);
        result
    }
}

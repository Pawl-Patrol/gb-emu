use crate::{
    constants::{
        FLAG_CARRY, FLAG_HALF_CARRY, FLAG_SUBTRACT, FLAG_ZERO, INTERRUPT_ENABLE, INTERRUPT_FLAG,
    },
    context::Context,
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

    pub halted: bool,
    pub interrupts_enabled: bool,
    pub pending_interrupt: Option<bool>,
}

/**
 * TODO:
 * - make inc_8bit take place using reference and pass a let mut value reference for hl
 * - extract some logic to apu.rs
 * - move constant to the corresponding files, e.g. SCANLINE to gpu.rs
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
            halted: false,
            interrupts_enabled: false,
            pending_interrupt: None,
        }
    }

    pub fn execute_next_opcode(&mut self, ctx: &mut Context) -> u16 {
        let cycles = if self.halted {
            4
        } else {
            let opcode = CPU::read_immediate_byte(ctx);
            let result = self.execute(ctx, opcode);
            result
        };

        if self.pending_interrupt == Some(true) && ctx.mmu.read_byte(ctx, self.pc - 1) != 0xFB {
            self.interrupts_enabled = true;
            self.pending_interrupt = None;
        } else if self.pending_interrupt == Some(false)
            && ctx.mmu.read_byte(ctx, self.pc - 1) != 0xF3
        {
            self.interrupts_enabled = false;
            self.pending_interrupt = None;
        }

        cycles
    }

    pub fn do_interrupts(&mut self, ctx: &mut Context) {
        if self.interrupts_enabled {
            let request = ctx.mmu.read_byte(ctx, INTERRUPT_FLAG);
            let enabled = ctx.mmu.read_byte(ctx, INTERRUPT_ENABLE);
            for i in 0..5 {
                if request.test_bit(i) && enabled.test_bit(i) {
                    self.service_interrupt(ctx, i);
                }
            }
        }
    }

    fn service_interrupt(&mut self, ctx: &mut Context, id: u8) {
        self.halted = false;
        self.interrupts_enabled = false;
        let mut request = ctx.mmu.read_byte(ctx, INTERRUPT_FLAG);
        request.reset_bit(id);
        ctx.mmu.write_byte(ctx, INTERRUPT_FLAG, request);

        self.push_stack(ctx, self.pc);

        match id {
            0 => self.pc = 0x40,
            1 => self.pc = 0x48,
            2 => self.pc = 0x50,
            3 => self.pc = 0x58,
            4 => self.pc = 0x60,
            _ => panic!("Invalid interrupt id"),
        }
    }

    pub fn request_interrupt(&mut self, ctx: &mut Context, id: u8) {
        let mut request = ctx.mmu.read_byte(ctx, INTERRUPT_FLAG);
        request.set_bit(id);
        ctx.mmu.write_byte(ctx, INTERRUPT_FLAG, request);
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

    fn read_immediate_byte(ctx: &mut Context) -> u8 {
        let result = ctx.mmu.read_byte(ctx, ctx.cpu.pc);
        ctx.cpu.pc = ctx.cpu.pc.wrapping_add(1);
        result
    }

    fn read_immediate_word(ctx: &mut Context) -> u16 {
        let lo = CPU::read_immediate_byte(ctx);
        let hi = CPU::read_immediate_byte(ctx);
        u16::from_bytes(hi, lo)
    }

    fn load_8bit(reg: &mut u8, byte: u8) -> u16 {
        *reg = byte;
        8
    }

    fn load_reg(&mut self, ctx: &Context, dest: &mut u8, src: &mut u8) -> u16 {
        *dest = *src;
        4
    }

    fn write_reg_into(ctx: &mut Context, addr: u16, reg: &mut u8) -> u16 {
        ctx.mmu.write_byte(ctx, addr, *reg);
        8
    }

    fn read_into_reg(ctx: &Context, addr: u16, reg: &mut u8) -> u16 {
        *reg = ctx.mmu.read_byte(ctx, addr);
        8
    }

    fn push_stack(&mut self, ctx: &mut Context, data: u16) -> u16 {
        ctx.mmu.write_byte(ctx, self.sp.wrapping_sub(1), data.hi());
        ctx.mmu.write_byte(ctx, self.sp.wrapping_sub(2), data.lo());
        self.sp = self.sp.wrapping_sub(2);
        16
    }

    fn pop_stack(&mut self, ctx: &mut Context) -> u16 {
        let lo = ctx.mmu.read_byte(ctx, self.sp);
        self.sp = self.sp.wrapping_add(1);
        let hi = ctx.mmu.read_byte(ctx, self.sp);
        u16::from_bytes(hi, lo)
    }

    pub fn execute(&mut self, ctx: &mut Context, opcode: u8) -> u16 {
        macro_rules! load {
            ($lhs: ident, $rhs: ident) => {{
                self.$lhs = self.$rhs;
                4
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
        match opcode {
            0x00 => 4, // NOP

            // 8-bit loads
            0x06 => CPU::load_8bit(&mut self.b, CPU::read_immediate_byte(ctx)), // LD B, n
            0x0E => CPU::load_8bit(&mut self.c, CPU::read_immediate_byte(ctx)), // LD C, n
            0x16 => CPU::load_8bit(&mut self.d, CPU::read_immediate_byte(ctx)), // LD D, n
            0x1E => CPU::load_8bit(&mut self.e, CPU::read_immediate_byte(ctx)), // LD E, n
            0x26 => CPU::load_8bit(&mut self.h, CPU::read_immediate_byte(ctx)), // LD H, n
            0x2E => CPU::load_8bit(&mut self.l, CPU::read_immediate_byte(ctx)), // LD L, n

            // load register
            0x7F => load!(a, a), // LD A, A
            0x78 => load!(a, b), // LD A, B
            0x79 => load!(a, c), // LD A, C
            0x7A => load!(a, d), // LD A, D
            0x7B => load!(a, e), // LD A, E
            0x7C => load!(a, h), // LD A, H
            0x7D => load!(a, l), // LD A, L

            0x47 => load!(b, a), // LD B, A
            0x40 => load!(b, b), // LD B, B
            0x41 => load!(b, c), // LD B, C
            0x42 => load!(b, d), // LD B, D
            0x43 => load!(b, e), // LD B, E
            0x44 => load!(b, h), // LD B, H
            0x45 => load!(b, l), // LD B, L

            0x4F => load!(c, a), // LD C, A
            0x48 => load!(c, b), // LD C, B
            0x49 => load!(c, c), // LD C, C
            0x4A => load!(c, d), // LD C, D
            0x4B => load!(c, e), // LD C, E
            0x4C => load!(c, h), // LD C, H
            0x4D => load!(c, l), // LD C, L

            0x57 => load!(d, a), // LD D, A
            0x50 => load!(d, b), // LD D, B
            0x51 => load!(d, c), // LD D, C
            0x52 => load!(d, d), // LD D, D
            0x53 => load!(d, e), // LD D, E
            0x54 => load!(d, h), // LD D, H
            0x55 => load!(d, l), // LD D, L

            0x5F => load!(e, a), // LD E, A
            0x58 => load!(e, b), // LD E, B
            0x59 => load!(e, c), // LD E, C
            0x5A => load!(e, d), // LD E, D
            0x5B => load!(e, e), // LD E, E
            0x5C => load!(e, h), // LD E, H
            0x5D => load!(e, l), // LD E, L

            0x67 => load!(h, a), // LD H, A
            0x60 => load!(h, b), // LD H, B
            0x61 => load!(h, c), // LD H, C
            0x62 => load!(h, d), // LD H, D
            0x63 => load!(h, e), // LD H, E
            0x64 => load!(h, h), // LD H, H
            0x65 => load!(h, l), // LD H, L

            0x6F => load!(l, a), // LD L, A
            0x68 => load!(l, b), // LD L, B
            0x69 => load!(l, c), // LD L, C
            0x6A => load!(l, d), // LD L, D
            0x6B => load!(l, e), // LD L, E
            0x6C => load!(l, h), // LD L, H
            0x6D => load!(l, l), // LD L, L

            // write register to memory
            0x70 => CPU::write_reg_into(ctx, self.hl(), &mut self.b),
            0x71 => CPU::write_reg_into(ctx, self.hl(), &mut self.c),
            0x72 => CPU::write_reg_into(ctx, self.hl(), &mut self.d),
            0x73 => CPU::write_reg_into(ctx, self.hl(), &mut self.e),
            0x74 => CPU::write_reg_into(ctx, self.hl(), &mut self.h),
            0x75 => CPU::write_reg_into(ctx, self.hl(), &mut self.l),

            // write memory to register
            0x7E => CPU::read_into_reg(ctx, self.hl(), &mut self.a),
            0x46 => CPU::read_into_reg(ctx, self.hl(), &mut self.b),
            0x4E => CPU::read_into_reg(ctx, self.hl(), &mut self.c),
            0x56 => CPU::read_into_reg(ctx, self.hl(), &mut self.d),
            0x5E => CPU::read_into_reg(ctx, self.hl(), &mut self.e),
            0x66 => CPU::read_into_reg(ctx, self.hl(), &mut self.h),
            0x6E => CPU::read_into_reg(ctx, self.hl(), &mut self.l),
            0x0A => CPU::read_into_reg(ctx, self.bc(), &mut self.a),
            0x1A => CPU::read_into_reg(ctx, self.de(), &mut self.a),
            0xF2 => CPU::read_into_reg(ctx, u16::from_bytes(0xFF, self.c), &mut self.a),

            // put a into memory address
            0x02 => CPU::write_reg_into(ctx, self.bc(), &mut self.a),
            0x12 => CPU::write_reg_into(ctx, self.de(), &mut self.a),
            0x77 => CPU::write_reg_into(ctx, self.hl(), &mut self.a),
            0xE2 => CPU::write_reg_into(ctx, u16::from_bytes(0xFF, self.c), &mut self.a),

            // put memory into a, decrement/increment HL
            0x3A => CPU::read_into_reg(ctx, self.hl(), &mut self.a) + op_16bit!(h, l, wrapping_add),
            0x2A => CPU::read_into_reg(ctx, self.hl(), &mut self.a) + op_16bit!(h, l, wrapping_add),

            // put a into memory, decrement/increment memory
            0x32 => {
                CPU::write_reg_into(ctx, self.hl(), &mut self.a) + op_16bit!(h, l, wrapping_sub)
            }
            0x22 => {
                CPU::write_reg_into(ctx, self.hl(), &mut self.a) + op_16bit!(h, l, wrapping_add)
            }

            // 16 bit loads
            0x01 => load_16bit!(b, c, CPU::read_immediate_word(ctx)),
            0x11 => load_16bit!(d, e, CPU::read_immediate_word(ctx)),
            0x21 => load_16bit!(h, l, CPU::read_immediate_word(ctx)),

            0x31 => {
                self.sp = CPU::read_immediate_word(ctx);
                12
            }
            0xF9 => {
                self.sp = self.hl();
                8
            }

            // push word onto stack
            0xF5 => self.push_stack(ctx, self.af()),
            0xC5 => self.push_stack(ctx, self.bc()),
            0xD5 => self.push_stack(ctx, self.de()),
            0xE5 => self.push_stack(ctx, self.hl()),

            // // pop word from stack into register
            0xF1 => {
                let word = self.pop_stack(ctx);
                self.a = word.hi();
                self.f = word.lo() & 0xF0;
                12
            }
            0xC1 => load_16bit!(b, c, self.pop_stack(ctx)),
            0xD1 => load_16bit!(d, e, self.pop_stack(ctx)),
            0xE1 => load_16bit!(h, l, self.pop_stack(ctx)),

            // 8-bit add
            0x87 => self.add_8bit(ctx, Some(self.a)),
            0x80 => self.add_8bit(ctx, Some(self.b)),
            0x81 => self.add_8bit(ctx, Some(self.c)),
            0x82 => self.add_8bit(ctx, Some(self.d)),
            0x83 => self.add_8bit(ctx, Some(self.e)),
            0x84 => self.add_8bit(ctx, Some(self.h)),
            0x85 => self.add_8bit(ctx, Some(self.l)),
            0x86 => self.add_8bit(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xC6 => self.add_8bit(ctx, None) + 4,

            // 8-bit add + carry
            0x8F => self.add_8bit_carry(ctx, Some(self.a)),
            0x88 => self.add_8bit_carry(ctx, Some(self.b)),
            0x89 => self.add_8bit_carry(ctx, Some(self.c)),
            0x8A => self.add_8bit_carry(ctx, Some(self.d)),
            0x8B => self.add_8bit_carry(ctx, Some(self.e)),
            0x8C => self.add_8bit_carry(ctx, Some(self.h)),
            0x8D => self.add_8bit_carry(ctx, Some(self.l)),
            0x8E => self.add_8bit_carry(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xCE => self.add_8bit_carry(ctx, None) + 4,

            // 8-bit subtract
            0x97 => self.sub_8bit(ctx, Some(self.a)),
            0x90 => self.sub_8bit(ctx, Some(self.b)),
            0x91 => self.sub_8bit(ctx, Some(self.c)),
            0x92 => self.sub_8bit(ctx, Some(self.d)),
            0x93 => self.sub_8bit(ctx, Some(self.e)),
            0x94 => self.sub_8bit(ctx, Some(self.h)),
            0x95 => self.sub_8bit(ctx, Some(self.l)),
            0x96 => self.sub_8bit(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xD6 => self.sub_8bit(ctx, None) + 4,

            // 8-bit subtract + carry
            0x9F => self.sub_8bit_carry(ctx, Some(self.a)),
            0x98 => self.sub_8bit_carry(ctx, Some(self.b)),
            0x99 => self.sub_8bit_carry(ctx, Some(self.c)),
            0x9A => self.sub_8bit_carry(ctx, Some(self.d)),
            0x9B => self.sub_8bit_carry(ctx, Some(self.e)),
            0x9C => self.sub_8bit_carry(ctx, Some(self.h)),
            0x9D => self.sub_8bit_carry(ctx, Some(self.l)),
            0x9E => self.sub_8bit_carry(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xDE => self.sub_8bit_carry(ctx, None) + 4,

            // 8-bit AND
            0xA7 => self.and_8bit(ctx, Some(self.a)),
            0xA0 => self.and_8bit(ctx, Some(self.b)),
            0xA1 => self.and_8bit(ctx, Some(self.c)),
            0xA2 => self.and_8bit(ctx, Some(self.d)),
            0xA3 => self.and_8bit(ctx, Some(self.e)),
            0xA4 => self.and_8bit(ctx, Some(self.h)),
            0xA5 => self.and_8bit(ctx, Some(self.l)),
            0xA6 => self.and_8bit(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xE6 => self.and_8bit(ctx, None) + 4,

            // 8-bit OR
            0xB7 => self.or_8bit(ctx, Some(self.a)),
            0xB0 => self.or_8bit(ctx, Some(self.b)),
            0xB1 => self.or_8bit(ctx, Some(self.c)),
            0xB2 => self.or_8bit(ctx, Some(self.d)),
            0xB3 => self.or_8bit(ctx, Some(self.e)),
            0xB4 => self.or_8bit(ctx, Some(self.h)),
            0xB5 => self.or_8bit(ctx, Some(self.l)),
            0xB6 => self.or_8bit(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xF6 => self.or_8bit(ctx, None) + 4,

            // 8-bit XOR
            0xAF => self.xor_8bit(ctx, Some(self.a)),
            0xA8 => self.xor_8bit(ctx, Some(self.b)),
            0xA9 => self.xor_8bit(ctx, Some(self.c)),
            0xAA => self.xor_8bit(ctx, Some(self.d)),
            0xAB => self.xor_8bit(ctx, Some(self.e)),
            0xAC => self.xor_8bit(ctx, Some(self.h)),
            0xAD => self.xor_8bit(ctx, Some(self.l)),
            0xAE => self.xor_8bit(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xEE => self.xor_8bit(ctx, None) + 4,

            // 8-bit compare
            0xBF => self.compare_8bit(ctx, Some(self.a)),
            0xB8 => self.compare_8bit(ctx, Some(self.b)),
            0xB9 => self.compare_8bit(ctx, Some(self.c)),
            0xBA => self.compare_8bit(ctx, Some(self.d)),
            0xBB => self.compare_8bit(ctx, Some(self.e)),
            0xBC => self.compare_8bit(ctx, Some(self.h)),
            0xBD => self.compare_8bit(ctx, Some(self.l)),
            0xBE => self.compare_8bit(ctx, Some(ctx.mmu.read_byte(ctx, self.hl()))) + 4,
            0xFE => self.compare_8bit(ctx, None) + 4,

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
                ctx.mmu
                    .write_byte(ctx, hl, self.inc_8bit(ctx.mmu.read_byte(ctx, hl)));
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
                ctx.mmu
                    .write_byte(ctx, hl, self.dec_8bit(ctx.mmu.read_byte(ctx, hl)));
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
            0xC3 => self.jump(ctx, 0, false, false),
            0xC2 => self.jump(ctx, FLAG_ZERO, true, false),
            0xCA => self.jump(ctx, FLAG_ZERO, true, true),
            0xD2 => self.jump(ctx, FLAG_CARRY, true, false),
            0xDA => self.jump(ctx, FLAG_CARRY, true, true),

            // jump with immediate data
            0x18 => self.jump_immediate(ctx, 0, false, false),
            0x20 => self.jump_immediate(ctx, FLAG_ZERO, true, false),
            0x28 => self.jump_immediate(ctx, FLAG_ZERO, true, true),
            0x30 => self.jump_immediate(ctx, FLAG_CARRY, true, false),
            0x38 => self.jump_immediate(ctx, FLAG_CARRY, true, true),

            // call
            0xCD => self.call(ctx, 0, false, false),
            0xC4 => self.call(ctx, FLAG_ZERO, true, false),
            0xCC => self.call(ctx, FLAG_ZERO, true, true),
            0xD4 => self.call(ctx, FLAG_CARRY, true, false),
            0xDC => self.call(ctx, FLAG_CARRY, true, true),

            // return
            0xC9 => self.return_from_call(ctx, 0, false, false),
            0xC0 => self.return_from_call(ctx, FLAG_ZERO, true, false),
            0xC8 => self.return_from_call(ctx, FLAG_ZERO, true, true),
            0xD0 => self.return_from_call(ctx, FLAG_CARRY, true, false),
            0xD8 => self.return_from_call(ctx, FLAG_CARRY, true, true),

            // restart
            0xC7 => self.restart(ctx, 0x00),
            0xCF => self.restart(ctx, 0x08),
            0xD7 => self.restart(ctx, 0x10),
            0xDF => self.restart(ctx, 0x18),
            0xE7 => self.restart(ctx, 0x20),
            0xEF => self.restart(ctx, 0x28),
            0xF7 => self.restart(ctx, 0x30),
            0xFF => self.restart(ctx, 0x38),

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

            0x07 => CPU::rotate_left_carry(ctx, &mut self.a, false),
            0x0F => CPU::rotate_right_carry(ctx, &mut self.a, false),
            0x17 => CPU::rotate_left(ctx, &mut self.a, false),
            0x1F => CPU::rotate_right(ctx, &mut self.a, false),

            0xD9 => {
                self.pc = self.pop_stack(ctx);
                self.interrupts_enabled = true;
                8
            }

            0x08 => {
                let address = CPU::read_immediate_word(ctx);
                ctx.mmu.write_byte(ctx, address, self.sp.lo());
                ctx.mmu
                    .write_byte(ctx, address.wrapping_add(1), self.sp.hi());
                20
            }

            0x36 => {
                let byte = CPU::read_immediate_byte(ctx);
                ctx.mmu.write_byte(ctx, self.hl(), byte);
                12
            }

            0xFA => {
                let address = CPU::read_immediate_word(ctx);
                self.a = ctx.mmu.read_byte(ctx, address);
                16
            }

            0x3E => {
                self.a = CPU::read_immediate_byte(ctx);
                8
            }

            0xEA => {
                let address = CPU::read_immediate_word(ctx);
                ctx.mmu.write_byte(ctx, address, self.a);
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
                let address = u16::from_bytes(0xFF, CPU::read_immediate_byte(ctx));
                ctx.mmu.write_byte(ctx, address, self.a);
                12
            }

            0xF0 => {
                let address = u16::from_bytes(0xFF, CPU::read_immediate_byte(ctx));
                self.a = ctx.mmu.read_byte(ctx, address);
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
                let offset = CPU::read_immediate_byte(ctx) as i8 as i16 as u16;
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
                let offset = CPU::read_immediate_byte(ctx) as i8 as i16 as u16;
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

            0xCB => self.execute_extended(ctx),

            0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => {
                panic!("Unknown opcode: {:02X}", opcode)
            }
        }
    }

    fn execute_extended(&mut self, ctx: &mut Context) -> u16 {
        let opcode = CPU::read_immediate_byte(ctx);

        macro_rules! perform_in_memory {
            ($action: expr, $value: ident) => {{
                let hl = self.hl();
                let mut $value = ctx.mmu.read_byte(ctx, hl);
                let cycles = $action;
                ctx.mmu.write_byte(ctx, hl, $value);
                cycles + 8
            }};
        }

        match opcode {
            // rotate left carry
            0x07 => CPU::rotate_left_carry(ctx, &mut self.a, true),
            0x00 => CPU::rotate_left_carry(ctx, &mut self.b, true),
            0x01 => CPU::rotate_left_carry(ctx, &mut self.c, true),
            0x02 => CPU::rotate_left_carry(ctx, &mut self.d, true),
            0x03 => CPU::rotate_left_carry(ctx, &mut self.e, true),
            0x04 => CPU::rotate_left_carry(ctx, &mut self.h, true),
            0x05 => CPU::rotate_left_carry(ctx, &mut self.l, true),
            0x06 => perform_in_memory!(CPU::rotate_left_carry(ctx, &mut value, true), value),

            // rotate right carry
            0x0F => CPU::rotate_right_carry(ctx, &mut self.a, true),
            0x08 => CPU::rotate_right_carry(ctx, &mut self.b, true),
            0x09 => CPU::rotate_right_carry(ctx, &mut self.c, true),
            0x0A => CPU::rotate_right_carry(ctx, &mut self.d, true),
            0x0B => CPU::rotate_right_carry(ctx, &mut self.e, true),
            0x0C => CPU::rotate_right_carry(ctx, &mut self.h, true),
            0x0D => CPU::rotate_right_carry(ctx, &mut self.l, true),
            0x0E => perform_in_memory!(CPU::rotate_right_carry(ctx, &mut value, true), value),

            // rotate left
            0x17 => CPU::rotate_left(ctx, &mut self.a, true),
            0x10 => CPU::rotate_left(ctx, &mut self.b, true),
            0x11 => CPU::rotate_left(ctx, &mut self.c, true),
            0x12 => CPU::rotate_left(ctx, &mut self.d, true),
            0x13 => CPU::rotate_left(ctx, &mut self.e, true),
            0x14 => CPU::rotate_left(ctx, &mut self.h, true),
            0x15 => CPU::rotate_left(ctx, &mut self.l, true),
            0x16 => perform_in_memory!(CPU::rotate_left(ctx, &mut value, true), value),

            // rotate right
            0x1F => CPU::rotate_right(ctx, &mut self.a, true),
            0x18 => CPU::rotate_right(ctx, &mut self.b, true),
            0x19 => CPU::rotate_right(ctx, &mut self.c, true),
            0x1A => CPU::rotate_right(ctx, &mut self.d, true),
            0x1B => CPU::rotate_right(ctx, &mut self.e, true),
            0x1C => CPU::rotate_right(ctx, &mut self.h, true),
            0x1D => CPU::rotate_right(ctx, &mut self.l, true),
            0x1E => perform_in_memory!(CPU::rotate_right(ctx, &mut value, true), value),

            // shift left arithmetic
            0x27 => CPU::shift_left_arithmetic(ctx, &mut self.a),
            0x20 => CPU::shift_left_arithmetic(ctx, &mut self.b),
            0x21 => CPU::shift_left_arithmetic(ctx, &mut self.c),
            0x22 => CPU::shift_left_arithmetic(ctx, &mut self.d),
            0x23 => CPU::shift_left_arithmetic(ctx, &mut self.e),
            0x24 => CPU::shift_left_arithmetic(ctx, &mut self.h),
            0x25 => CPU::shift_left_arithmetic(ctx, &mut self.l),
            0x26 => perform_in_memory!(CPU::shift_left_arithmetic(ctx, &mut value), value),

            // shift right arithmetic
            0x2F => CPU::shift_right_arithmetic(ctx, &mut self.a),
            0x28 => CPU::shift_right_arithmetic(ctx, &mut self.b),
            0x29 => CPU::shift_right_arithmetic(ctx, &mut self.c),
            0x2A => CPU::shift_right_arithmetic(ctx, &mut self.d),
            0x2B => CPU::shift_right_arithmetic(ctx, &mut self.e),
            0x2C => CPU::shift_right_arithmetic(ctx, &mut self.h),
            0x2D => CPU::shift_right_arithmetic(ctx, &mut self.l),
            0x2E => perform_in_memory!(CPU::shift_right_arithmetic(ctx, &mut value), value),

            // shift right logical
            0x3F => CPU::shift_right_logical(ctx, &mut self.a),
            0x38 => CPU::shift_right_logical(ctx, &mut self.b),
            0x39 => CPU::shift_right_logical(ctx, &mut self.c),
            0x3A => CPU::shift_right_logical(ctx, &mut self.d),
            0x3B => CPU::shift_right_logical(ctx, &mut self.e),
            0x3C => CPU::shift_right_logical(ctx, &mut self.h),
            0x3D => CPU::shift_right_logical(ctx, &mut self.l),
            0x3E => perform_in_memory!(CPU::shift_right_logical(ctx, &mut value), value),

            // swap nibbles
            0x37 => CPU::swap_nibbles(ctx, &mut self.a),
            0x30 => CPU::swap_nibbles(ctx, &mut self.b),
            0x31 => CPU::swap_nibbles(ctx, &mut self.c),
            0x32 => CPU::swap_nibbles(ctx, &mut self.d),
            0x33 => CPU::swap_nibbles(ctx, &mut self.e),
            0x34 => CPU::swap_nibbles(ctx, &mut self.h),
            0x35 => CPU::swap_nibbles(ctx, &mut self.l),
            0x36 => perform_in_memory!(CPU::swap_nibbles(ctx, &mut value), value),

            // test bit 0
            0x47 => CPU::test_bit(ctx, &self.a, 0),
            0x40 => CPU::test_bit(ctx, &self.b, 0),
            0x41 => CPU::test_bit(ctx, &self.c, 0),
            0x42 => CPU::test_bit(ctx, &self.d, 0),
            0x43 => CPU::test_bit(ctx, &self.e, 0),
            0x44 => CPU::test_bit(ctx, &self.h, 0),
            0x45 => CPU::test_bit(ctx, &self.l, 0),
            0x46 => perform_in_memory!(CPU::test_bit(ctx, &value, 0), value),

            // test bit 1
            0x4F => CPU::test_bit(ctx, &self.a, 1),
            0x48 => CPU::test_bit(ctx, &self.b, 1),
            0x49 => CPU::test_bit(ctx, &self.c, 1),
            0x4A => CPU::test_bit(ctx, &self.d, 1),
            0x4B => CPU::test_bit(ctx, &self.e, 1),
            0x4C => CPU::test_bit(ctx, &self.h, 1),
            0x4D => CPU::test_bit(ctx, &self.l, 1),
            0x4E => perform_in_memory!(CPU::test_bit(ctx, &value, 1), value),

            // test bit 2
            0x57 => CPU::test_bit(ctx, &self.a, 2),
            0x50 => CPU::test_bit(ctx, &self.b, 2),
            0x51 => CPU::test_bit(ctx, &self.c, 2),
            0x52 => CPU::test_bit(ctx, &self.d, 2),
            0x53 => CPU::test_bit(ctx, &self.e, 2),
            0x54 => CPU::test_bit(ctx, &self.h, 2),
            0x55 => CPU::test_bit(ctx, &self.l, 2),
            0x56 => perform_in_memory!(CPU::test_bit(ctx, &value, 2), value),

            // test bit 3
            0x5F => CPU::test_bit(ctx, &self.a, 3),
            0x58 => CPU::test_bit(ctx, &self.b, 3),
            0x59 => CPU::test_bit(ctx, &self.c, 3),
            0x5A => CPU::test_bit(ctx, &self.d, 3),
            0x5B => CPU::test_bit(ctx, &self.e, 3),
            0x5C => CPU::test_bit(ctx, &self.h, 3),
            0x5D => CPU::test_bit(ctx, &self.l, 3),
            0x5E => perform_in_memory!(CPU::test_bit(ctx, &value, 3), value),

            // test bit 4
            0x67 => CPU::test_bit(ctx, &self.a, 4),
            0x60 => CPU::test_bit(ctx, &self.b, 4),
            0x61 => CPU::test_bit(ctx, &self.c, 4),
            0x62 => CPU::test_bit(ctx, &self.d, 4),
            0x63 => CPU::test_bit(ctx, &self.e, 4),
            0x64 => CPU::test_bit(ctx, &self.h, 4),
            0x65 => CPU::test_bit(ctx, &self.l, 4),
            0x66 => perform_in_memory!(CPU::test_bit(ctx, &value, 4), value),

            // test bit 5
            0x6F => CPU::test_bit(ctx, &self.a, 5),
            0x68 => CPU::test_bit(ctx, &self.b, 5),
            0x69 => CPU::test_bit(ctx, &self.c, 5),
            0x6A => CPU::test_bit(ctx, &self.d, 5),
            0x6B => CPU::test_bit(ctx, &self.e, 5),
            0x6C => CPU::test_bit(ctx, &self.h, 5),
            0x6D => CPU::test_bit(ctx, &self.l, 5),
            0x6E => perform_in_memory!(CPU::test_bit(ctx, &value, 5), value),

            // test bit 6
            0x77 => CPU::test_bit(ctx, &self.a, 6),
            0x70 => CPU::test_bit(ctx, &self.b, 6),
            0x71 => CPU::test_bit(ctx, &self.c, 6),
            0x72 => CPU::test_bit(ctx, &self.d, 6),
            0x73 => CPU::test_bit(ctx, &self.e, 6),
            0x74 => CPU::test_bit(ctx, &self.h, 6),
            0x75 => CPU::test_bit(ctx, &self.l, 6),
            0x76 => perform_in_memory!(CPU::test_bit(ctx, &value, 6), value),

            // test bit 7
            0x7F => CPU::test_bit(ctx, &self.a, 7),
            0x78 => CPU::test_bit(ctx, &self.b, 7),
            0x79 => CPU::test_bit(ctx, &self.c, 7),
            0x7A => CPU::test_bit(ctx, &self.d, 7),
            0x7B => CPU::test_bit(ctx, &self.e, 7),
            0x7C => CPU::test_bit(ctx, &self.h, 7),
            0x7D => CPU::test_bit(ctx, &self.l, 7),
            0x7E => perform_in_memory!(CPU::test_bit(ctx, &value, 7), value),

            // reset bit 0
            0x87 => CPU::reset_bit(&mut self.a, 0),
            0x80 => CPU::reset_bit(&mut self.b, 0),
            0x81 => CPU::reset_bit(&mut self.c, 0),
            0x82 => CPU::reset_bit(&mut self.d, 0),
            0x83 => CPU::reset_bit(&mut self.e, 0),
            0x84 => CPU::reset_bit(&mut self.h, 0),
            0x85 => CPU::reset_bit(&mut self.l, 0),
            0x86 => perform_in_memory!(CPU::reset_bit(&mut value, 0), value),

            // reset bit 1
            0x8F => CPU::reset_bit(&mut self.a, 1),
            0x88 => CPU::reset_bit(&mut self.b, 1),
            0x89 => CPU::reset_bit(&mut self.c, 1),
            0x8A => CPU::reset_bit(&mut self.d, 1),
            0x8B => CPU::reset_bit(&mut self.e, 1),
            0x8C => CPU::reset_bit(&mut self.h, 1),
            0x8D => CPU::reset_bit(&mut self.l, 1),
            0x8E => perform_in_memory!(CPU::reset_bit(&mut value, 1), value),

            // reset bit 2
            0x97 => CPU::reset_bit(&mut self.a, 2),
            0x90 => CPU::reset_bit(&mut self.b, 2),
            0x91 => CPU::reset_bit(&mut self.c, 2),
            0x92 => CPU::reset_bit(&mut self.d, 2),
            0x93 => CPU::reset_bit(&mut self.e, 2),
            0x94 => CPU::reset_bit(&mut self.h, 2),
            0x95 => CPU::reset_bit(&mut self.l, 2),
            0x96 => perform_in_memory!(CPU::reset_bit(&mut value, 2), value),

            // reset bit 3
            0x9F => CPU::reset_bit(&mut self.a, 3),
            0x98 => CPU::reset_bit(&mut self.b, 3),
            0x99 => CPU::reset_bit(&mut self.c, 3),
            0x9A => CPU::reset_bit(&mut self.d, 3),
            0x9B => CPU::reset_bit(&mut self.e, 3),
            0x9C => CPU::reset_bit(&mut self.h, 3),
            0x9D => CPU::reset_bit(&mut self.l, 3),
            0x9E => perform_in_memory!(CPU::reset_bit(&mut value, 3), value),

            // reset bit 4
            0xA7 => CPU::reset_bit(&mut self.a, 4),
            0xA0 => CPU::reset_bit(&mut self.b, 4),
            0xA1 => CPU::reset_bit(&mut self.c, 4),
            0xA2 => CPU::reset_bit(&mut self.d, 4),
            0xA3 => CPU::reset_bit(&mut self.e, 4),
            0xA4 => CPU::reset_bit(&mut self.h, 4),
            0xA5 => CPU::reset_bit(&mut self.l, 4),
            0xA6 => perform_in_memory!(CPU::reset_bit(&mut value, 4), value),

            // reset bit 5
            0xAF => CPU::reset_bit(&mut self.a, 5),
            0xA8 => CPU::reset_bit(&mut self.b, 5),
            0xA9 => CPU::reset_bit(&mut self.c, 5),
            0xAA => CPU::reset_bit(&mut self.d, 5),
            0xAB => CPU::reset_bit(&mut self.e, 5),
            0xAC => CPU::reset_bit(&mut self.h, 5),
            0xAD => CPU::reset_bit(&mut self.l, 5),
            0xAE => perform_in_memory!(CPU::reset_bit(&mut value, 5), value),

            // reset bit 6
            0xB7 => CPU::reset_bit(&mut self.a, 6),
            0xB0 => CPU::reset_bit(&mut self.b, 6),
            0xB1 => CPU::reset_bit(&mut self.c, 6),
            0xB2 => CPU::reset_bit(&mut self.d, 6),
            0xB3 => CPU::reset_bit(&mut self.e, 6),
            0xB4 => CPU::reset_bit(&mut self.h, 6),
            0xB5 => CPU::reset_bit(&mut self.l, 6),
            0xB6 => perform_in_memory!(CPU::reset_bit(&mut value, 6), value),

            // reset bit 7
            0xBF => CPU::reset_bit(&mut self.a, 7),
            0xB8 => CPU::reset_bit(&mut self.b, 7),
            0xB9 => CPU::reset_bit(&mut self.c, 7),
            0xBA => CPU::reset_bit(&mut self.d, 7),
            0xBB => CPU::reset_bit(&mut self.e, 7),
            0xBC => CPU::reset_bit(&mut self.h, 7),
            0xBD => CPU::reset_bit(&mut self.l, 7),
            0xBE => perform_in_memory!(CPU::reset_bit(&mut value, 7), value),

            // set bit 0
            0xC7 => CPU::set_bit(&mut self.a, 0),
            0xC0 => CPU::set_bit(&mut self.b, 0),
            0xC1 => CPU::set_bit(&mut self.c, 0),
            0xC2 => CPU::set_bit(&mut self.d, 0),
            0xC3 => CPU::set_bit(&mut self.e, 0),
            0xC4 => CPU::set_bit(&mut self.h, 0),
            0xC5 => CPU::set_bit(&mut self.l, 0),
            0xC6 => perform_in_memory!(CPU::set_bit(&mut value, 0), value),

            // set bit 1
            0xCF => CPU::set_bit(&mut self.a, 1),
            0xC8 => CPU::set_bit(&mut self.b, 1),
            0xC9 => CPU::set_bit(&mut self.c, 1),
            0xCA => CPU::set_bit(&mut self.d, 1),
            0xCB => CPU::set_bit(&mut self.e, 1),
            0xCC => CPU::set_bit(&mut self.h, 1),
            0xCD => CPU::set_bit(&mut self.l, 1),
            0xCE => perform_in_memory!(CPU::set_bit(&mut value, 1), value),

            // set bit 2
            0xD7 => CPU::set_bit(&mut self.a, 2),
            0xD0 => CPU::set_bit(&mut self.b, 2),
            0xD1 => CPU::set_bit(&mut self.c, 2),
            0xD2 => CPU::set_bit(&mut self.d, 2),
            0xD3 => CPU::set_bit(&mut self.e, 2),
            0xD4 => CPU::set_bit(&mut self.h, 2),
            0xD5 => CPU::set_bit(&mut self.l, 2),
            0xD6 => perform_in_memory!(CPU::set_bit(&mut value, 2), value),

            // set bit 3
            0xDF => CPU::set_bit(&mut self.a, 3),
            0xD8 => CPU::set_bit(&mut self.b, 3),
            0xD9 => CPU::set_bit(&mut self.c, 3),
            0xDA => CPU::set_bit(&mut self.d, 3),
            0xDB => CPU::set_bit(&mut self.e, 3),
            0xDC => CPU::set_bit(&mut self.h, 3),
            0xDD => CPU::set_bit(&mut self.l, 3),
            0xDE => perform_in_memory!(CPU::set_bit(&mut value, 3), value),

            // set bit 4
            0xE7 => CPU::set_bit(&mut self.a, 4),
            0xE0 => CPU::set_bit(&mut self.b, 4),
            0xE1 => CPU::set_bit(&mut self.c, 4),
            0xE2 => CPU::set_bit(&mut self.d, 4),
            0xE3 => CPU::set_bit(&mut self.e, 4),
            0xE4 => CPU::set_bit(&mut self.h, 4),
            0xE5 => CPU::set_bit(&mut self.l, 4),
            0xE6 => perform_in_memory!(CPU::set_bit(&mut value, 4), value),

            // set bit 5
            0xEF => CPU::set_bit(&mut self.a, 5),
            0xE8 => CPU::set_bit(&mut self.b, 5),
            0xE9 => CPU::set_bit(&mut self.c, 5),
            0xEA => CPU::set_bit(&mut self.d, 5),
            0xEB => CPU::set_bit(&mut self.e, 5),
            0xEC => CPU::set_bit(&mut self.h, 5),
            0xED => CPU::set_bit(&mut self.l, 5),
            0xEE => perform_in_memory!(CPU::set_bit(&mut value, 5), value),

            // set bit 6
            0xF7 => CPU::set_bit(&mut self.a, 6),
            0xF0 => CPU::set_bit(&mut self.b, 6),
            0xF1 => CPU::set_bit(&mut self.c, 6),
            0xF2 => CPU::set_bit(&mut self.d, 6),
            0xF3 => CPU::set_bit(&mut self.e, 6),
            0xF4 => CPU::set_bit(&mut self.h, 6),
            0xF5 => CPU::set_bit(&mut self.l, 6),
            0xF6 => perform_in_memory!(CPU::set_bit(&mut value, 6), value),

            // set bit 7
            0xFF => CPU::set_bit(&mut self.a, 7),
            0xF8 => CPU::set_bit(&mut self.b, 7),
            0xF9 => CPU::set_bit(&mut self.c, 7),
            0xFA => CPU::set_bit(&mut self.d, 7),
            0xFB => CPU::set_bit(&mut self.e, 7),
            0xFC => CPU::set_bit(&mut self.h, 7),
            0xFD => CPU::set_bit(&mut self.l, 7),
            0xFE => perform_in_memory!(CPU::set_bit(&mut value, 7), value),
        }
    }

    fn jump(&mut self, ctx: &mut Context, flag: u8, use_condition: bool, condition: bool) -> u16 {
        let address = CPU::read_immediate_word(ctx);
        if !use_condition || self.f.test_bit(flag) == condition {
            self.pc = address;
            return 16;
        }
        12
    }

    fn jump_immediate(
        &mut self,
        ctx: &mut Context,
        flag: u8,
        use_condition: bool,
        condition: bool,
    ) -> u16 {
        let offset = CPU::read_immediate_byte(ctx) as i8 as i16;
        if !use_condition || self.f.test_bit(flag) == condition {
            self.pc = self.pc.wrapping_add_signed(offset);
            return 12;
        }
        8
    }

    fn call(&mut self, ctx: &mut Context, flag: u8, use_condition: bool, condition: bool) -> u16 {
        let address = CPU::read_immediate_word(ctx);
        if !use_condition || self.f.test_bit(flag) == condition {
            self.push_stack(ctx, self.pc);
            self.pc = address;
            return 24;
        }
        12
    }

    fn restart(&mut self, ctx: &mut Context, offset: u16) -> u16 {
        self.push_stack(ctx, self.pc);
        self.pc = offset;
        32
    }

    fn return_from_call(
        &mut self,
        ctx: &mut Context,
        flag: u8,
        use_condition: bool,
        condition: bool,
    ) -> u16 {
        if !use_condition || self.f.test_bit(flag) == condition {
            self.pc = self.pop_stack(ctx);
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

    fn add_8bit(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
        let (result, carry) = self.a.overflowing_add(value);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, (self.a & 0x0f) + (value & 0x0f) > 0x0f);
        self.f.toggle_bit(FLAG_CARRY, carry);
        self.a = result;
        4
    }

    fn add_8bit_carry(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
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

    fn sub_8bit(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
        let (result, carry) = self.a.overflowing_sub(value);
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.set_bit(FLAG_SUBTRACT);
        self.f
            .toggle_bit(FLAG_HALF_CARRY, (self.a & 0x0f) < (value & 0x0f));
        self.f.toggle_bit(FLAG_CARRY, carry);
        self.a = result;
        4
    }

    fn sub_8bit_carry(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
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

    fn and_8bit(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
        self.a &= value;
        self.f.toggle_bit(FLAG_ZERO, self.a == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.set_bit(FLAG_HALF_CARRY);
        self.f.reset_bit(FLAG_CARRY);
        4
    }

    fn or_8bit(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
        let result = self.a | value;
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.reset_bit(FLAG_HALF_CARRY);
        self.f.reset_bit(FLAG_CARRY);
        self.a = result;
        4
    }

    fn xor_8bit(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
        let result = self.a ^ value;
        self.f.toggle_bit(FLAG_ZERO, result == 0);
        self.f.reset_bit(FLAG_SUBTRACT);
        self.f.reset_bit(FLAG_HALF_CARRY);
        self.f.reset_bit(FLAG_CARRY);
        self.a = result;
        4
    }

    fn compare_8bit(&mut self, ctx: &mut Context, value: Option<u8>) -> u16 {
        let value = value.unwrap_or_else(|| CPU::read_immediate_byte(ctx));
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

    fn rotate_left(ctx: &mut Context, value: &mut u8, set_zero: bool) -> u16 {
        let ci = ctx.cpu.f.test_bit(FLAG_CARRY) as u8;
        let co = *value & 0x80;
        *value = (*value << 1) | ci;
        ctx.cpu.f.toggle_bit(FLAG_ZERO, (*value == 0) && set_zero);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        ctx.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }

    fn rotate_left_carry(ctx: &mut Context, value: &mut u8, set_zero: bool) -> u16 {
        let co = *value & 0x80;
        *value = (*value).rotate_left(1);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, (*value == 0) && set_zero);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        ctx.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }

    fn rotate_right(ctx: &mut Context, value: &mut u8, set_zero: bool) -> u16 {
        let ci = ctx.cpu.f.test_bit(FLAG_CARRY) as u8;
        let co = *value & 0x01;
        *value = (*value >> 1) | (ci << 7);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, (*value == 0) && set_zero);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        ctx.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }

    fn rotate_right_carry(ctx: &mut Context, value: &mut u8, set_zero: bool) -> u16 {
        let co = *value & 0x01;
        *value = (*value).rotate_right(1);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, (*value == 0) && set_zero);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        ctx.cpu.f.toggle_bit(FLAG_CARRY, co != 0);
        8
    }

    fn shift_left_arithmetic(ctx: &mut Context, value: &mut u8) -> u16 {
        let is_msb_set = (*value).test_bit(7);
        *value <<= 1;
        ctx.cpu.f.toggle_bit(FLAG_CARRY, is_msb_set);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, *value == 0);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }

    fn shift_right_arithmetic(ctx: &mut Context, value: &mut u8) -> u16 {
        let is_lsb_set = (*value).test_bit(0);
        let is_msb_set = (*value).test_bit(7);
        *value >>= 1;
        value.toggle_bit(7, is_msb_set);
        ctx.cpu.f.toggle_bit(FLAG_CARRY, is_lsb_set);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, *value == 0);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }

    fn shift_right_logical(ctx: &mut Context, value: &mut u8) -> u16 {
        let is_lsb_set = (*value).test_bit(0);
        *value >>= 1;
        ctx.cpu.f.toggle_bit(FLAG_CARRY, is_lsb_set);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, *value == 0);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }

    fn swap_nibbles(ctx: &mut Context, value: &mut u8) -> u16 {
        *value = (*value << 4) | (*value >> 4);
        ctx.cpu.f.toggle_bit(FLAG_ZERO, *value == 0);
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.reset_bit(FLAG_HALF_CARRY);
        ctx.cpu.f.reset_bit(FLAG_CARRY);
        8
    }

    fn test_bit(ctx: &mut Context, value: &u8, bit: u8) -> u16 {
        ctx.cpu.f.toggle_bit(FLAG_ZERO, !value.test_bit(bit));
        ctx.cpu.f.reset_bit(FLAG_SUBTRACT);
        ctx.cpu.f.set_bit(FLAG_HALF_CARRY);
        8
    }

    fn set_bit(value: &mut u8, bit: u8) -> u16 {
        value.set_bit(bit);
        8
    }

    fn reset_bit(value: &mut u8, bit: u8) -> u16 {
        value.reset_bit(bit);
        8
    }
}

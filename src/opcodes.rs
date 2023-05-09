use crate::{
    constants::{FLAG_CARRY, FLAG_HALF_CARRY, FLAG_SUBTRACT, FLAG_ZERO},
    emulator::Emulator,
    traits::{Register, SetBit, TestBit, ToggleBit},
};

/*
    Reference file:
    #include "Config.h"
#include "Emulator.h"
#include <stdio.h>

void Emulator::ExecuteOpcode(BYTE opcode)
{
    switch(opcode)
    {
        //no-op
        case 0x00: m_CyclesThisUpdate+=4 ; break ;

        // 8-Bit Loads
        case 0x06: CPU_8BIT_LOAD(m_RegisterBC.hi) ; break ;
        case 0x0E: CPU_8BIT_LOAD(m_RegisterBC.lo) ; break ;
        case 0x16: CPU_8BIT_LOAD(m_RegisterDE.hi) ; break ;
        case 0x1E: CPU_8BIT_LOAD(m_RegisterDE.lo) ; break ;
        case 0x26: CPU_8BIT_LOAD(m_RegisterHL.hi) ; break ;
        case 0x2E: CPU_8BIT_LOAD(m_RegisterHL.lo) ; break ;

        // load reg
        case 0x7F: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterAF.hi, 4) ; break ;
        case 0x78: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterBC.hi, 4) ; break ;
        case 0x79: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterBC.lo, 4) ; break ;
        case 0x7A: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterDE.hi, 4) ; break ;
        case 0x7B: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterDE.lo, 4) ; break ;
        case 0x7C: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterHL.hi, 4) ; break ;
        case 0x7D: CPU_REG_LOAD(m_RegisterAF.hi, m_RegisterHL.lo, 4) ; break ;
        case 0x40: CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterBC.hi, 4) ; break ;
        case 0x41: CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterBC.lo, 4) ; break ;
        case 0x42: CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterDE.hi, 4) ; break ;
        case 0x43: CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterDE.lo, 4) ; break ;
        case 0x44: CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterHL.hi, 4) ; break ;
        case 0x45: CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterHL.lo, 4) ; break ;
        case 0x48: CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterBC.hi, 4) ; break ;
        case 0x49: CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterBC.lo, 4) ; break ;
        case 0x4A: CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterDE.hi, 4) ; break ;
        case 0x4B: CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterDE.lo, 4) ; break ;
        case 0x4C: CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterHL.hi, 4) ; break ;
        case 0x4D: CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterHL.lo, 4) ; break ;
        case 0x50: CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterBC.hi, 4) ; break ;
        case 0x51: CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterBC.lo, 4) ; break ;
        case 0x52: CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterDE.hi, 4) ; break ;
        case 0x53: CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterDE.lo, 4) ; break ;
        case 0x54: CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterHL.hi, 4) ; break ;
        case 0x55: CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterHL.lo, 4) ; break ;
        case 0x58: CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterBC.hi, 4) ; break ;
        case 0x59: CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterBC.lo, 4) ; break ;
        case 0x5A: CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterDE.hi, 4) ; break ;
        case 0x5B: CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterDE.lo, 4) ; break ;
        case 0x5C: CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterHL.hi, 4) ; break ;
        case 0x5D: CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterHL.lo, 4) ; break ;
        case 0x60: CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterBC.hi, 4) ; break ;
        case 0x61: CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterBC.lo, 4) ; break ;
        case 0x62: CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterDE.hi, 4) ; break ;
        case 0x63: CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterDE.lo, 4) ; break ;
        case 0x64: CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterHL.hi, 4) ; break ;
        case 0x65: CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterHL.lo, 4) ; break ;
        case 0x68: CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterBC.hi, 4) ; break ;
        case 0x69: CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterBC.lo, 4) ; break ;
        case 0x6A: CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterDE.hi, 4) ; break ;
        case 0x6B: CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterDE.lo, 4) ; break ;
        case 0x6C: CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterHL.hi, 4) ; break ;
        case 0x6D: CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterHL.lo, 4) ; break ;

        // write reg to memory
        case 0x70: WriteByte(m_RegisterHL.reg, m_RegisterBC.hi) ; m_CyclesThisUpdate+=8; break ;
        case 0x71: WriteByte(m_RegisterHL.reg, m_RegisterBC.lo) ; m_CyclesThisUpdate+=8;break ;
        case 0x72: WriteByte(m_RegisterHL.reg, m_RegisterDE.hi) ; m_CyclesThisUpdate+=8;break ;
        case 0x73: WriteByte(m_RegisterHL.reg, m_RegisterDE.lo) ; m_CyclesThisUpdate+=8;break ;
        case 0x74: WriteByte(m_RegisterHL.reg, m_RegisterHL.hi) ; m_CyclesThisUpdate+=8;break ;
        case 0x75: WriteByte(m_RegisterHL.reg, m_RegisterHL.lo) ; m_CyclesThisUpdate+=8;break ;

        // write memory to reg
        case 0x7E: CPU_REG_LOAD_ROM(m_RegisterAF.hi, m_RegisterHL.reg) ; break ;
        case 0x46: CPU_REG_LOAD_ROM(m_RegisterBC.hi, m_RegisterHL.reg) ; break ;
        case 0x4E: CPU_REG_LOAD_ROM(m_RegisterBC.lo, m_RegisterHL.reg) ; break ;
        case 0x56: CPU_REG_LOAD_ROM(m_RegisterDE.hi, m_RegisterHL.reg) ; break ;
        case 0x5E: CPU_REG_LOAD_ROM(m_RegisterDE.lo, m_RegisterHL.reg) ; break ;
        case 0x66: CPU_REG_LOAD_ROM(m_RegisterHL.hi, m_RegisterHL.reg) ; break ;
        case 0x6E: CPU_REG_LOAD_ROM(m_RegisterHL.lo, m_RegisterHL.reg) ; break ;
        case 0x0A: CPU_REG_LOAD_ROM(m_RegisterAF.hi, m_RegisterBC.reg) ; break ;
        case 0x1A: CPU_REG_LOAD_ROM(m_RegisterAF.hi, m_RegisterDE.reg) ; break ;
        case 0xF2: CPU_REG_LOAD_ROM(m_RegisterAF.hi, (0xFF00+m_RegisterBC.lo)) ; break ;



        // put a into register
        case 0x47 : CPU_REG_LOAD(m_RegisterBC.hi, m_RegisterAF.hi, 4) ; break ;
        case 0x4F : CPU_REG_LOAD(m_RegisterBC.lo, m_RegisterAF.hi, 4) ; break ;
        case 0x57 : CPU_REG_LOAD(m_RegisterDE.hi, m_RegisterAF.hi, 4) ; break ;
        case 0x5F : CPU_REG_LOAD(m_RegisterDE.lo, m_RegisterAF.hi, 4) ; break ;
        case 0x67 : CPU_REG_LOAD(m_RegisterHL.hi, m_RegisterAF.hi, 4) ; break ;
        case 0x6F : CPU_REG_LOAD(m_RegisterHL.lo, m_RegisterAF.hi, 4) ; break ;

        // put a into memory address
        case 0x02: WriteByte(m_RegisterBC.reg, m_RegisterAF.hi) ; m_CyclesThisUpdate+=8; break ;
        case 0x12: WriteByte(m_RegisterDE.reg, m_RegisterAF.hi) ; m_CyclesThisUpdate+=8; break ;
        case 0x77: WriteByte(m_RegisterHL.reg, m_RegisterAF.hi) ; m_CyclesThisUpdate+=8; break ;
        case 0xE2: WriteByte((0xFF00+m_RegisterBC.lo), m_RegisterAF.hi) ; m_CyclesThisUpdate+=8; break ;

        // put memory into a, decrement/increment memory
        case 0x3A: CPU_REG_LOAD_ROM(m_RegisterAF.hi,m_RegisterHL.reg ) ; CPU_16BIT_DEC(m_RegisterHL.reg,0) ;break ;
        case 0x2A: CPU_REG_LOAD_ROM(m_RegisterAF.hi,m_RegisterHL.reg ) ; CPU_16BIT_INC(m_RegisterHL.reg,0) ;break ;

        // put a into memory, decrement/increment reg
        case 0x32: WriteByte(m_RegisterHL.reg, m_RegisterAF.hi); CPU_16BIT_DEC(m_RegisterHL.reg,0) ; m_CyclesThisUpdate += 8; break;
        case 0x22: WriteByte(m_RegisterHL.reg, m_RegisterAF.hi); CPU_16BIT_INC(m_RegisterHL.reg,0) ; m_CyclesThisUpdate += 8;break;

        // 16 bit loads
        case 0x01: CPU_16BIT_LOAD( m_RegisterBC.reg ); break ;
        case 0x11: CPU_16BIT_LOAD( m_RegisterDE.reg );break ;
        case 0x21: CPU_16BIT_LOAD( m_RegisterHL.reg );break ;
        case 0x31: CPU_16BIT_LOAD( m_StackPointer.reg );break ;
        case 0xF9: m_StackPointer.reg = m_RegisterHL.reg ; m_CyclesThisUpdate+=8; break ;

        // push word onto stack
        case 0xF5: PushWordOntoStack( m_RegisterAF.reg ) ; m_CyclesThisUpdate+=16 ;break;
        case 0xC5: PushWordOntoStack( m_RegisterBC.reg ) ; m_CyclesThisUpdate+=16 ;break;
        case 0xD5: PushWordOntoStack( m_RegisterDE.reg ) ; m_CyclesThisUpdate+=16 ;break;
        case 0xE5: PushWordOntoStack( m_RegisterHL.reg ) ; m_CyclesThisUpdate+=16 ; break;

        // pop word from stack into reg
        case 0xF1: m_RegisterAF.reg = PopWordOffStack( ) ; m_CyclesThisUpdate+=12 ;break;
        case 0xC1: m_RegisterBC.reg = PopWordOffStack( ) ; m_CyclesThisUpdate+=12 ;break;
        case 0xD1: m_RegisterDE.reg = PopWordOffStack( ) ; m_CyclesThisUpdate+=12 ;break;
        case 0xE1: m_RegisterHL.reg = PopWordOffStack( ) ; m_CyclesThisUpdate+=12 ; break;

        // 8-bit add
        case 0x87: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterAF.hi,4,false,false) ; break ;
        case 0x80: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterBC.hi,4,false,false) ; break ;
        case 0x81: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterBC.lo,4,false,false) ; break ;
        case 0x82: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterDE.hi,4,false,false) ; break ;
        case 0x83: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterDE.lo,4,false,false) ; break ;
        case 0x84: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterHL.hi,4,false,false) ; break ;
        case 0x85: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterHL.lo,4,false,false) ; break ;
        case 0x86: CPU_8BIT_ADD(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8,false,false) ; break ;
        case 0xC6: CPU_8BIT_ADD(m_RegisterAF.hi, 0,8,true,false) ; break ;

        // 8-bit add + carry
        case 0x8F: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterAF.hi,4,false,true) ; break ;
        case 0x88: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterBC.hi,4,false,true) ; break ;
        case 0x89: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterBC.lo,4,false,true) ; break ;
        case 0x8A: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterDE.hi,4,false,true) ; break ;
        case 0x8B: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterDE.lo,4,false,true) ; break ;
        case 0x8C: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterHL.hi,4,false,true) ; break ;
        case 0x8D: CPU_8BIT_ADD(m_RegisterAF.hi, m_RegisterHL.lo,4,false,true) ; break ;
        case 0x8E: CPU_8BIT_ADD(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8,false,true) ; break ;
        case 0xCE: CPU_8BIT_ADD(m_RegisterAF.hi, 0,8,true,true) ; break ;

        // 8-bit subtract
        case 0x97: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterAF.hi,4,false,false) ; break ;
        case 0x90: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterBC.hi,4,false,false) ; break ;
        case 0x91: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterBC.lo,4,false,false) ; break ;
        case 0x92: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterDE.hi,4,false,false) ; break ;
        case 0x93: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterDE.lo,4,false,false) ; break ;
        case 0x94: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterHL.hi,4,false,false) ; break ;
        case 0x95: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterHL.lo,4,false,false) ; break ;
        case 0x96: CPU_8BIT_SUB(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8,false,false) ; break ;
        case 0xD6: CPU_8BIT_SUB(m_RegisterAF.hi, 0,8,true,false) ; break ;

        // 8-bit subtract + carry
        case 0x9F: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterAF.hi,4,false,true) ; break ;
        case 0x98: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterBC.hi,4,false,true) ; break ;
        case 0x99: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterBC.lo,4,false,true) ; break ;
        case 0x9A: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterDE.hi,4,false,true) ; break ;
        case 0x9B: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterDE.lo,4,false,true) ; break ;
        case 0x9C: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterHL.hi,4,false,true) ; break ;
        case 0x9D: CPU_8BIT_SUB(m_RegisterAF.hi, m_RegisterHL.lo,4,false,true) ; break ;
        case 0x9E: CPU_8BIT_SUB(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8,false,true) ; break ;
        case 0xDE: CPU_8BIT_SUB(m_RegisterAF.hi, 0,8,true,true) ; break ;

        // 8-bit AND reg with reg
        case 0xA7: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterAF.hi,4, false) ; break ;
        case 0xA0: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterBC.hi,4, false) ; break ;
        case 0xA1: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterBC.lo,4, false) ; break ;
        case 0xA2: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterDE.hi,4, false) ; break ;
        case 0xA3: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterDE.lo,4, false) ; break ;
        case 0xA4: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterHL.hi,4, false) ; break ;
        case 0xA5: CPU_8BIT_AND(m_RegisterAF.hi, m_RegisterHL.lo,4, false) ; break ;
        case 0xA6: CPU_8BIT_AND(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8, false) ; break ;
        case 0xE6: CPU_8BIT_AND(m_RegisterAF.hi, 0,8, true) ; break ;

        // 8-bit OR reg with reg
        case 0xB7: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterAF.hi,4, false) ; break ;
        case 0xB0: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterBC.hi,4, false) ; break ;
        case 0xB1: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterBC.lo,4, false) ; break ;
        case 0xB2: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterDE.hi,4, false) ; break ;
        case 0xB3: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterDE.lo,4, false) ; break ;
        case 0xB4: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterHL.hi,4, false) ; break ;
        case 0xB5: CPU_8BIT_OR(m_RegisterAF.hi, m_RegisterHL.lo,4, false) ; break ;
        case 0xB6: CPU_8BIT_OR(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8, false) ; break ;
        case 0xF6: CPU_8BIT_OR(m_RegisterAF.hi, 0,8, true) ; break ;

        // 8-bit XOR reg with reg
        case 0xAF: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterAF.hi,4, false) ; break ;
        case 0xA8: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterBC.hi,4, false) ; break ;
        case 0xA9: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterBC.lo,4, false) ; break ;
        case 0xAA: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterDE.hi,4, false) ; break ;
        case 0xAB: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterDE.lo,4, false) ; break ;
        case 0xAC: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterHL.hi,4, false) ; break ;
        case 0xAD: CPU_8BIT_XOR(m_RegisterAF.hi, m_RegisterHL.lo,4, false) ; break ;
        case 0xAE: CPU_8BIT_XOR(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8, false) ; break ;
        case 0xEE: CPU_8BIT_XOR(m_RegisterAF.hi, 0,8, true) ; break ;

        // 8-Bit compare
        case 0xBF: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterAF.hi,4, false) ; break ;
        case 0xB8: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterBC.hi,4, false) ; break ;
        case 0xB9: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterBC.lo,4, false) ; break ;
        case 0xBA: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterDE.hi,4, false) ; break ;
        case 0xBB: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterDE.lo,4, false) ; break ;
        case 0xBC: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterHL.hi,4, false) ; break ;
        case 0xBD: CPU_8BIT_COMPARE(m_RegisterAF.hi, m_RegisterHL.lo,4, false) ; break ;
        case 0xBE: CPU_8BIT_COMPARE(m_RegisterAF.hi, ReadMemory(m_RegisterHL.reg),8, false) ; break ;
        case 0xFE: CPU_8BIT_COMPARE(m_RegisterAF.hi, 0,8, true) ; break ;

        // 8-bit inc
        case 0x3C: CPU_8BIT_INC(m_RegisterAF.hi,4); break ;
        case 0x04: CPU_8BIT_INC(m_RegisterBC.hi,4); break ;
        case 0x0C: CPU_8BIT_INC(m_RegisterBC.lo,4); break ;
        case 0x14: CPU_8BIT_INC(m_RegisterDE.hi,4); break ;
        case 0x1C: CPU_8BIT_INC(m_RegisterDE.lo,4); break ;
        case 0x24: CPU_8BIT_INC(m_RegisterHL.hi,4); break ;
        case 0x2C: CPU_8BIT_INC(m_RegisterHL.lo,4); break ;
        case 0x34: CPU_8BIT_MEMORY_INC(m_RegisterHL.reg,12); break ;

        // 8-bit dec
        case 0x3D: CPU_8BIT_DEC(m_RegisterAF.hi,4); break ;
        case 0x05: CPU_8BIT_DEC(m_RegisterBC.hi,4); break ;
        case 0x0D: CPU_8BIT_DEC(m_RegisterBC.lo,4); break ;
        case 0x15: CPU_8BIT_DEC(m_RegisterDE.hi,4); break ;
        case 0x1D: CPU_8BIT_DEC(m_RegisterDE.lo,4); break ;
        case 0x25: CPU_8BIT_DEC(m_RegisterHL.hi,4); break ;
        case 0x2D: CPU_8BIT_DEC(m_RegisterHL.lo,4); break ;
        case 0x35: CPU_8BIT_MEMORY_DEC(m_RegisterHL.reg,12); break ;

        // 16-bit add
        case 0x09: CPU_16BIT_ADD(m_RegisterHL.reg,m_RegisterBC.reg,8) ; break ;
        case 0x19: CPU_16BIT_ADD(m_RegisterHL.reg,m_RegisterDE.reg,8) ; break ;
        case 0x29: CPU_16BIT_ADD(m_RegisterHL.reg,m_RegisterHL.reg,8) ; break ;
        case 0x39: CPU_16BIT_ADD(m_RegisterHL.reg,m_StackPointer.reg,8) ; break ;

        // inc 16-bit register
        case 0x03: CPU_16BIT_INC( m_RegisterBC.reg, 8) ; break ;
        case 0x13: CPU_16BIT_INC( m_RegisterDE.reg, 8) ; break ;
        case 0x23: CPU_16BIT_INC( m_RegisterHL.reg, 8) ; break ;
        case 0x33: CPU_16BIT_INC( m_StackPointer.reg, 8) ; break ;

        // dec 16-bit register
        case 0x0B: CPU_16BIT_DEC( m_RegisterBC.reg, 8) ; break ;
        case 0x1B: CPU_16BIT_DEC( m_RegisterDE.reg, 8) ; break ;
        case 0x2B: CPU_16BIT_DEC( m_RegisterHL.reg, 8) ; break ;
        case 0x3B: CPU_16BIT_DEC( m_StackPointer.reg, 8) ; break ;

        // jumps
        case 0xE9: m_CyclesThisUpdate+=4 ; m_ProgramCounter = m_RegisterHL.reg ; break ;
        case 0xC3: CPU_JUMP(false, 0, false) ; break ;
        case 0xC2: CPU_JUMP(true, FLAG_Z, false) ; break ;
        case 0xCA: CPU_JUMP(true, FLAG_Z, true) ; break ;
        case 0xD2: CPU_JUMP(true, FLAG_C, false) ; break ;
        case 0xDA: CPU_JUMP(true, FLAG_C, true) ; break ;

        // jump with immediate data
        case 0x18 : CPU_JUMP_IMMEDIATE( false, 0, false ) ; break ;
        case 0x20 : CPU_JUMP_IMMEDIATE( true, FLAG_Z, false ) ;break ;
        case 0x28 : CPU_JUMP_IMMEDIATE( true, FLAG_Z, true ) ;break ;
        case 0x30 : CPU_JUMP_IMMEDIATE( true, FLAG_C, false) ;break ;
        case 0x38 : CPU_JUMP_IMMEDIATE( true, FLAG_C, true ) ;break ;

        // calls
        case 0xCD : CPU_CALL( false, 0, false) ; break ;
        case 0xC4 : CPU_CALL( true, FLAG_Z, false) ;break ;
        case 0xCC : CPU_CALL( true, FLAG_Z, true) ;break ;
        case 0xD4 : CPU_CALL( true, FLAG_C, false) ;break ;
        case 0xDC : CPU_CALL( true, FLAG_C, true) ; break ;

        // returns
        case 0xC9: CPU_RETURN( false, 0, false ) ; break ;
        case 0xC0: CPU_RETURN( true, FLAG_Z, false ) ; break ;
        case 0xC8: CPU_RETURN( true, FLAG_Z, true ) ; break ;
        case 0xD0: CPU_RETURN( true, FLAG_C, false ) ; break ;
        case 0xD8: CPU_RETURN( true, FLAG_C, true ) ; break ;


        // restarts
        case 0xC7: CPU_RESTARTS( 0x00 ) ; break ;
        case 0xCF: CPU_RESTARTS( 0x08 ) ; break ;
        case 0xD7: CPU_RESTARTS( 0x10 ) ; break ;
        case 0xDF: CPU_RESTARTS( 0x18 ) ; break ;
        case 0xE7: CPU_RESTARTS( 0x20 ) ; break ;
        case 0xEF: CPU_RESTARTS( 0x28 ) ; break ;
        case 0xF7: CPU_RESTARTS( 0x30 ) ; break ;
        case 0xFF: CPU_RESTARTS( 0x38 ) ; break ;

        case 0x27: CPU_DAA( ) ; break ;

        // handle the extended opcodes
        case 0xCB: ExecuteExtendedOpcode( ) ; break ;

        // unique instructions
        case 0x07:CPU_RLC(m_RegisterAF.hi); break ;
        case 0x0F:CPU_RRC(m_RegisterAF.hi) ;	break ;
        case 0x17:CPU_RL(m_RegisterAF.hi) ; break ;
        case 0x1F:CPU_RR(m_RegisterAF.hi) ;	break ;

        case 0xD9:
        {
            //LOGMESSAGE(Logging::MSG_INFO, "Returning from iterupt") ;
            m_ProgramCounter = PopWordOffStack( ) ;
            m_EnableInterupts = true ;
            m_CyclesThisUpdate+=8 ;
            LogMessage::GetSingleton()->DoLogMessage("Returning from interupt", false) ;
        }break ;

        case 0x08:
        {
            WORD nn = ReadWord( ) ;
            m_ProgramCounter+=2 ;
            WriteByte(nn, m_StackPointer.lo) ;
            nn++ ;
            WriteByte(nn, m_StackPointer.hi) ;
            m_CyclesThisUpdate += 20 ;
        }break ;

        case 0x36:
        {
            m_CyclesThisUpdate+=12 ;
            BYTE n = ReadMemory(m_ProgramCounter) ;
            m_ProgramCounter++;
            WriteByte(m_RegisterHL.reg, n) ;
        }break ;

        case 0xFA:
        {
            m_CyclesThisUpdate+=16 ;
            WORD nn = ReadWord( ) ;
            m_ProgramCounter+=2 ;
            BYTE n = ReadMemory(nn) ;
            m_RegisterAF.hi = n ;
        }break ;

        case 0x3E:
        {
            m_CyclesThisUpdate+=8;
            BYTE n = ReadMemory(m_ProgramCounter) ;
            m_ProgramCounter++ ;
            m_RegisterAF.hi = n;
        }break ;
        case 0xEA:
        {
            m_CyclesThisUpdate+=16 ;
            WORD nn = ReadWord( ) ;
            m_ProgramCounter+=2 ;
            WriteByte(nn, m_RegisterAF.hi) ;
        }break ;

        case 0xF3:
        {
            m_PendingInteruptDisabled = true ;
            m_CyclesThisUpdate+=4 ;
        }break ;

        case 0xFB:
        {
            m_PendingInteruptEnabled = true ;
            m_CyclesThisUpdate+=4 ;
        }break ;

        case 0xE0:
        {
            BYTE n = ReadMemory(m_ProgramCounter) ;
            m_ProgramCounter++ ;
            WORD address = 0xFF00 + n ;
            WriteByte(address, m_RegisterAF.hi) ;
            m_CyclesThisUpdate += 12 ;
        }break ;

        case 0xF0:
        {
            BYTE n = ReadMemory(m_ProgramCounter) ;
            m_ProgramCounter++ ;
            WORD address = 0xFF00 + n ;
            m_RegisterAF.hi = ReadMemory( address ) ;
            m_CyclesThisUpdate+=12 ;
        }break ;

        case 0x2F:
        {
            m_CyclesThisUpdate += 4;
            m_RegisterAF.hi ^= 0xFF;

            m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_N) ;
            m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
        }break ;

        case 0x76:
        {
            //LOGMESSAGE(Logging::MSG_INFO, "Halting cpu") ;
            m_CyclesThisUpdate += 4 ;
            m_Halted = true ;
        }break ;

        case 0x3F:
        {
            m_CyclesThisUpdate += 4 ;
            if (TestBit(m_RegisterAF.lo, FLAG_C))
                m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_C) ;
            else
                m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

            m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H) ;
            m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N) ;
        }break ;

        case 0x37:
        {
            m_CyclesThisUpdate += 4;
            m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C);
            m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H);
            m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N);
        } break ;

        case 0xF8:
        {
            SIGNED_BYTE n = ReadMemory(m_ProgramCounter) ;
            m_ProgramCounter++ ;
            m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_Z);
            m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N);


            WORD value = (m_StackPointer.reg + n) & 0xFFFF;

            m_RegisterHL.reg = value ;
            unsigned int v = m_StackPointer.reg + n ;

            if( n > 0xFFFF )
                m_RegisterAF.lo = BitSet(m_RegisterAF.lo,FLAG_C) ;
            else
                m_RegisterAF.lo = BitReset(m_RegisterAF.lo,FLAG_C) ;

            if( (m_StackPointer.reg & 0xF) + (n & 0xF) > 0xF )
                m_RegisterAF.lo = BitSet(m_RegisterAF.lo,FLAG_H) ;
            else
                m_RegisterAF.lo = BitReset(m_RegisterAF.lo,FLAG_H) ;

        }break ;

        case 0x10:
        {
            m_ProgramCounter++ ;
            m_CyclesThisUpdate+= 4 ;
        }break ;

        default:
        {
            char mybuf[200] ;
            sprintf(mybuf, "Unhandled Opcode %x", opcode) ;
            LogMessage::GetSingleton()->DoLogMessage(mybuf,true) ;
            assert(false) ;
        } break;
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

void Emulator::ExecuteExtendedOpcode( )
{
    BYTE opcode = m_Rom[m_ProgramCounter] ;

    if ((m_ProgramCounter >= 0x4000 && m_ProgramCounter <= 0x7FFF) || (m_ProgramCounter >= 0xA000 && m_ProgramCounter <= 0xBFFF))
        opcode = ReadMemory(m_ProgramCounter) ;

    if (false)
    {
        char buffer[200] ;
        sprintf(buffer, "EXTENDEDOP = %x PC = %x\n", opcode, m_ProgramCounter) ;
        LogMessage::GetSingleton()->DoLogMessage(buffer,false) ;
    }

    m_ProgramCounter++ ;

    //LOGMESSAGE(Logging::MSG_INFO, STR::Format("Processing Extended Opcode %x, Program Counter: %x", opcode, m_ProgramCounter).ConstCharPtr()) ;

    switch(opcode)
    {
        // rotate left through carry
          case 0x0 : CPU_RLC(m_RegisterBC.hi) ; break ;
          case 0x1 : CPU_RLC(m_RegisterBC.lo) ; break ;
          case 0x2 : CPU_RLC(m_RegisterDE.hi) ; break ;
          case 0x3 : CPU_RLC(m_RegisterDE.lo) ; break ;
          case 0x4 : CPU_RLC(m_RegisterHL.hi) ; break ;
          case 0x5 : CPU_RLC(m_RegisterHL.lo) ; break ;
          case 0x6 : CPU_RLC_MEMORY(m_RegisterHL.reg) ; break ;
          case 0x7 : CPU_RLC(m_RegisterAF.hi) ; break ;

          // rotate right through carry
          case 0x8 : CPU_RRC(m_RegisterBC.hi) ; break ;
          case 0x9 : CPU_RRC(m_RegisterBC.lo) ; break ;
          case 0xA : CPU_RRC(m_RegisterDE.hi) ; break ;
          case 0xB : CPU_RRC(m_RegisterDE.lo) ; break ;
          case 0xC : CPU_RRC(m_RegisterHL.hi) ; break ;
          case 0xD : CPU_RRC(m_RegisterHL.lo) ; break ;
          case 0xE : CPU_RRC_MEMORY(m_RegisterHL.reg) ; break ;
          case 0xF : CPU_RRC(m_RegisterAF.hi) ; break ;

          // rotate left
          case 0x10: CPU_RL(m_RegisterBC.hi); break;
          case 0x11: CPU_RL(m_RegisterBC.lo); break;
          case 0x12: CPU_RL(m_RegisterDE.hi); break;
          case 0x13: CPU_RL(m_RegisterDE.lo); break;
          case 0x14: CPU_RL(m_RegisterHL.hi); break;
          case 0x15: CPU_RL(m_RegisterHL.lo); break;
          case 0x16: CPU_RL_MEMORY(m_RegisterHL.reg); break;
          case 0x17: CPU_RL(m_RegisterAF.hi); break;

          // rotate right
          case 0x18: CPU_RR(m_RegisterBC.hi); break;
          case 0x19: CPU_RR(m_RegisterBC.lo); break;
          case 0x1A: CPU_RR(m_RegisterDE.hi); break;
          case 0x1B: CPU_RR(m_RegisterDE.lo); break;
          case 0x1C: CPU_RR(m_RegisterHL.hi); break;
          case 0x1D: CPU_RR(m_RegisterHL.lo); break;
          case 0x1E: CPU_RR_MEMORY(m_RegisterHL.reg); break;
          case 0x1F: CPU_RR(m_RegisterAF.hi); break;

          case 0x20 : CPU_SLA( m_RegisterBC.hi ) ;break ;
          case 0x21 : CPU_SLA( m_RegisterBC.lo ) ;break ;
          case 0x22 : CPU_SLA( m_RegisterDE.hi ) ;break ;
          case 0x23 : CPU_SLA( m_RegisterDE.lo ) ;break ;
          case 0x24 : CPU_SLA( m_RegisterHL.hi ) ;break ;
          case 0x25 : CPU_SLA( m_RegisterHL.lo ) ;break ;
          case 0x26 : CPU_SLA_MEMORY( m_RegisterHL.reg ) ;break ;
        case 0x27 : CPU_SLA( m_RegisterAF.hi ) ;break ;

          case 0x28 : CPU_SRA( m_RegisterBC.hi ) ; break ;
          case 0x29 : CPU_SRA( m_RegisterBC.lo ) ; break ;
          case 0x2A : CPU_SRA( m_RegisterDE.hi ) ; break ;
          case 0x2B : CPU_SRA( m_RegisterDE.lo ) ; break ;
          case 0x2C : CPU_SRA( m_RegisterHL.hi ) ; break ;
          case 0x2D : CPU_SRA( m_RegisterHL.lo ) ; break ;
          case 0x2E : CPU_SRA_MEMORY( m_RegisterHL.reg ) ; break ;
          case 0x2F : CPU_SRA( m_RegisterAF.hi ) ; break ;

          case 0x38 : CPU_SRL( m_RegisterBC.hi ) ; break ;
          case 0x39 : CPU_SRL( m_RegisterBC.lo ) ; break ;
          case 0x3A : CPU_SRL( m_RegisterDE.hi ) ; break ;
          case 0x3B : CPU_SRL( m_RegisterDE.lo ) ; break ;
          case 0x3C : CPU_SRL( m_RegisterHL.hi ) ; break ;
          case 0x3D : CPU_SRL( m_RegisterHL.lo ) ; break ;
          case 0x3E : CPU_SRL_MEMORY( m_RegisterHL.reg ) ; break ;
          case 0x3F : CPU_SRL( m_RegisterAF.hi ) ; break ;

            // swap nibbles
        case 0x37 : CPU_SWAP_NIBBLES( m_RegisterAF.hi ) ;break ;
        case 0x30 : CPU_SWAP_NIBBLES( m_RegisterBC.hi ) ;break ;
        case 0x31 : CPU_SWAP_NIBBLES( m_RegisterBC.lo ) ;break ;
        case 0x32 : CPU_SWAP_NIBBLES( m_RegisterDE.hi ) ;break ;
        case 0x33 : CPU_SWAP_NIBBLES( m_RegisterDE.lo ) ;break ;
        case 0x34 : CPU_SWAP_NIBBLES( m_RegisterHL.hi ) ;break ;
        case 0x35 : CPU_SWAP_NIBBLES( m_RegisterHL.lo ) ;break ;
        case 0x36 : CPU_SWAP_NIB_MEM( m_RegisterHL.reg ) ;break ;

        // test bit
        case 0x40 : CPU_TEST_BIT( m_RegisterBC.hi, 0 , 8 ) ; break ;
        case 0x41 : CPU_TEST_BIT( m_RegisterBC.lo, 0 , 8 ) ; break ;
        case 0x42 : CPU_TEST_BIT( m_RegisterDE.hi, 0 , 8 ) ; break ;
        case 0x43 : CPU_TEST_BIT( m_RegisterDE.lo, 0 , 8 ) ; break ;
        case 0x44 : CPU_TEST_BIT( m_RegisterHL.hi, 0 , 8 ) ; break ;
        case 0x45 : CPU_TEST_BIT( m_RegisterHL.lo, 0 , 8 ) ; break ;
        case 0x46 : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 0 , 16 ) ; break ;
        case 0x47 : CPU_TEST_BIT( m_RegisterAF.hi, 0 , 8 ) ; break ;
        case 0x48 : CPU_TEST_BIT( m_RegisterBC.hi, 1 , 8 ) ; break ;
        case 0x49 : CPU_TEST_BIT( m_RegisterBC.lo, 1 , 8 ) ; break ;
        case 0x4A : CPU_TEST_BIT( m_RegisterDE.hi, 1 , 8 ) ; break ;
        case 0x4B : CPU_TEST_BIT( m_RegisterDE.lo, 1 , 8 ) ; break ;
        case 0x4C : CPU_TEST_BIT( m_RegisterHL.hi, 1 , 8 ) ; break ;
        case 0x4D : CPU_TEST_BIT( m_RegisterHL.lo, 1 , 8 ) ; break ;
        case 0x4E : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 1 , 16 ) ; break ;
        case 0x4F : CPU_TEST_BIT( m_RegisterAF.hi, 1 , 8 ) ; break ;
        case 0x50 : CPU_TEST_BIT( m_RegisterBC.hi, 2 , 8 ) ; break ;
        case 0x51 : CPU_TEST_BIT( m_RegisterBC.lo, 2 , 8 ) ; break ;
        case 0x52 : CPU_TEST_BIT( m_RegisterDE.hi, 2 , 8 ) ; break ;
        case 0x53 : CPU_TEST_BIT( m_RegisterDE.lo, 2 , 8 ) ; break ;
        case 0x54 : CPU_TEST_BIT( m_RegisterHL.hi, 2 , 8 ) ; break ;
        case 0x55 : CPU_TEST_BIT( m_RegisterHL.lo, 2 , 8 ) ; break ;
        case 0x56 : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 2 , 16 ) ; break ;
        case 0x57 : CPU_TEST_BIT( m_RegisterAF.hi, 2 , 8 ) ; break ;
        case 0x58 : CPU_TEST_BIT( m_RegisterBC.hi, 3 , 8 ) ; break ;
        case 0x59 : CPU_TEST_BIT( m_RegisterBC.lo, 3 , 8 ) ; break ;
        case 0x5A : CPU_TEST_BIT( m_RegisterDE.hi, 3 , 8 ) ; break ;
        case 0x5B : CPU_TEST_BIT( m_RegisterDE.lo, 3 , 8 ) ; break ;
        case 0x5C : CPU_TEST_BIT( m_RegisterHL.hi, 3 , 8 ) ; break ;
        case 0x5D : CPU_TEST_BIT( m_RegisterHL.lo, 3 , 8 ) ; break ;
        case 0x5E : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 3 , 16 ) ; break ;
        case 0x5F : CPU_TEST_BIT( m_RegisterAF.hi, 3 , 8 ) ; break ;
        case 0x60 : CPU_TEST_BIT( m_RegisterBC.hi, 4 , 8 ) ; break ;
        case 0x61 : CPU_TEST_BIT( m_RegisterBC.lo, 4 , 8 ) ; break ;
        case 0x62 : CPU_TEST_BIT( m_RegisterDE.hi, 4 , 8 ) ; break ;
        case 0x63 : CPU_TEST_BIT( m_RegisterDE.lo, 4 , 8 ) ; break ;
        case 0x64 : CPU_TEST_BIT( m_RegisterHL.hi, 4 , 8 ) ; break ;
        case 0x65 : CPU_TEST_BIT( m_RegisterHL.lo, 4 , 8 ) ; break ;
        case 0x66 : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 4 , 16 ) ; break ;
        case 0x67 : CPU_TEST_BIT( m_RegisterAF.hi, 4 , 8 ) ; break ;
        case 0x68 : CPU_TEST_BIT( m_RegisterBC.hi, 5 , 8 ) ; break ;
        case 0x69 : CPU_TEST_BIT( m_RegisterBC.lo, 5 , 8 ) ; break ;
        case 0x6A : CPU_TEST_BIT( m_RegisterDE.hi, 5 , 8 ) ; break ;
        case 0x6B : CPU_TEST_BIT( m_RegisterDE.lo, 5 , 8 ) ; break ;
        case 0x6C : CPU_TEST_BIT( m_RegisterHL.hi, 5 , 8 ) ; break ;
        case 0x6D : CPU_TEST_BIT( m_RegisterHL.lo, 5 , 8 ) ; break ;
        case 0x6E : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 5 , 16 ) ; break ;
        case 0x6F : CPU_TEST_BIT( m_RegisterAF.hi, 5 , 8 ) ; break ;
        case 0x70 : CPU_TEST_BIT( m_RegisterBC.hi, 6 , 8 ) ; break ;
        case 0x71 : CPU_TEST_BIT( m_RegisterBC.lo, 6 , 8 ) ; break ;
        case 0x72 : CPU_TEST_BIT( m_RegisterDE.hi, 6 , 8 ) ; break ;
        case 0x73 : CPU_TEST_BIT( m_RegisterDE.lo, 6 , 8 ) ; break ;
        case 0x74 : CPU_TEST_BIT( m_RegisterHL.hi, 6 , 8 ) ; break ;
        case 0x75 : CPU_TEST_BIT( m_RegisterHL.lo, 6 , 8 ) ; break ;
        case 0x76 : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 6 , 16 ) ; break ;
        case 0x77 : CPU_TEST_BIT( m_RegisterAF.hi, 6 , 8 ) ; break ;
        case 0x78 : CPU_TEST_BIT( m_RegisterBC.hi, 7 , 8 ) ; break ;
        case 0x79 : CPU_TEST_BIT( m_RegisterBC.lo, 7 , 8 ) ; break ;
        case 0x7A : CPU_TEST_BIT( m_RegisterDE.hi, 7 , 8 ) ; break ;
        case 0x7B : CPU_TEST_BIT( m_RegisterDE.lo, 7 , 8 ) ; break ;
        case 0x7C : CPU_TEST_BIT( m_RegisterHL.hi, 7 , 8 ) ; break ;
        case 0x7D : CPU_TEST_BIT( m_RegisterHL.lo, 7 , 8 ) ; break ;
        case 0x7E : CPU_TEST_BIT( ReadMemory(m_RegisterHL.reg), 7 , 16 ) ; break ;
        case 0x7F : CPU_TEST_BIT( m_RegisterAF.hi, 7 , 8 ) ; break ;

        // reset bit
        case 0x80 : CPU_RESET_BIT( m_RegisterBC.hi, 0 ) ; break ;
        case 0x81 : CPU_RESET_BIT( m_RegisterBC.lo, 0 ) ; break ;
        case 0x82 : CPU_RESET_BIT( m_RegisterDE.hi, 0 ) ; break ;
        case 0x83 : CPU_RESET_BIT( m_RegisterDE.lo, 0 ) ; break ;
        case 0x84 : CPU_RESET_BIT( m_RegisterHL.hi, 0 ) ; break ;
        case 0x85 : CPU_RESET_BIT( m_RegisterHL.lo, 0 ) ; break ;
        case 0x86 : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 0 ) ; break ;
        case 0x87 : CPU_RESET_BIT( m_RegisterAF.hi, 0 ) ; break ;
        case 0x88 : CPU_RESET_BIT( m_RegisterBC.hi, 1  ) ; break ;
        case 0x89 : CPU_RESET_BIT( m_RegisterBC.lo, 1 ) ; break ;
        case 0x8A : CPU_RESET_BIT( m_RegisterDE.hi, 1 ) ; break ;
        case 0x8B : CPU_RESET_BIT( m_RegisterDE.lo, 1 ) ; break ;
        case 0x8C : CPU_RESET_BIT( m_RegisterHL.hi, 1 ) ; break ;
        case 0x8D : CPU_RESET_BIT( m_RegisterHL.lo, 1 ) ; break ;
        case 0x8E : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 1 ) ; break ;
        case 0x8F : CPU_RESET_BIT( m_RegisterAF.hi, 1  ) ; break ;
        case 0x90 : CPU_RESET_BIT( m_RegisterBC.hi, 2  ) ; break ;
        case 0x91 : CPU_RESET_BIT( m_RegisterBC.lo, 2  ) ; break ;
        case 0x92 : CPU_RESET_BIT( m_RegisterDE.hi, 2  ) ; break ;
        case 0x93 : CPU_RESET_BIT( m_RegisterDE.lo, 2  ) ; break ;
        case 0x94 : CPU_RESET_BIT( m_RegisterHL.hi, 2  ) ; break ;
        case 0x95 : CPU_RESET_BIT( m_RegisterHL.lo, 2  ) ; break ;
        case 0x96 : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 2 ) ; break ;
        case 0x97 : CPU_RESET_BIT( m_RegisterAF.hi, 2  ) ; break ;
        case 0x98 : CPU_RESET_BIT( m_RegisterBC.hi, 3  ) ; break ;
        case 0x99 : CPU_RESET_BIT( m_RegisterBC.lo, 3  ) ; break ;
        case 0x9A : CPU_RESET_BIT( m_RegisterDE.hi, 3  ) ; break ;
        case 0x9B : CPU_RESET_BIT( m_RegisterDE.lo, 3  ) ; break ;
        case 0x9C : CPU_RESET_BIT( m_RegisterHL.hi, 3  ) ; break ;
        case 0x9D : CPU_RESET_BIT( m_RegisterHL.lo, 3  ) ; break ;
        case 0x9E : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 3  ) ; break ;
        case 0x9F : CPU_RESET_BIT( m_RegisterAF.hi, 3  ) ; break ;
        case 0xA0 : CPU_RESET_BIT( m_RegisterBC.hi, 4  ) ; break ;
        case 0xA1 : CPU_RESET_BIT( m_RegisterBC.lo, 4  ) ; break ;
        case 0xA2 : CPU_RESET_BIT( m_RegisterDE.hi, 4  ) ; break ;
        case 0xA3 : CPU_RESET_BIT( m_RegisterDE.lo, 4  ) ; break ;
        case 0xA4 : CPU_RESET_BIT( m_RegisterHL.hi, 4  ) ; break ;
        case 0xA5 : CPU_RESET_BIT( m_RegisterHL.lo, 4  ) ; break ;
        case 0xA6 : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 4) ; break ;
        case 0xA7 : CPU_RESET_BIT( m_RegisterAF.hi, 4 ) ; break ;
        case 0xA8 : CPU_RESET_BIT( m_RegisterBC.hi, 5 ) ; break ;
        case 0xA9 : CPU_RESET_BIT( m_RegisterBC.lo, 5 ) ; break ;
        case 0xAA : CPU_RESET_BIT( m_RegisterDE.hi, 5 ) ; break ;
        case 0xAB : CPU_RESET_BIT( m_RegisterDE.lo, 5 ) ; break ;
        case 0xAC : CPU_RESET_BIT( m_RegisterHL.hi, 5 ) ; break ;
        case 0xAD : CPU_RESET_BIT( m_RegisterHL.lo, 5 ) ; break ;
        case 0xAE : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 5 ) ; break ;
        case 0xAF : CPU_RESET_BIT( m_RegisterAF.hi, 5  ) ; break ;
        case 0xB0 : CPU_RESET_BIT( m_RegisterBC.hi, 6  ) ; break ;
        case 0xB1 : CPU_RESET_BIT( m_RegisterBC.lo, 6  ) ; break ;
        case 0xB2 : CPU_RESET_BIT( m_RegisterDE.hi, 6  ) ; break ;
        case 0xB3 : CPU_RESET_BIT( m_RegisterDE.lo, 6  ) ; break ;
        case 0xB4 : CPU_RESET_BIT( m_RegisterHL.hi, 6  ) ; break ;
        case 0xB5 : CPU_RESET_BIT( m_RegisterHL.lo, 6  ) ; break ;
        case 0xB6 : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 6 ) ; break ;
        case 0xB7 : CPU_RESET_BIT( m_RegisterAF.hi, 6  ) ; break ;
        case 0xB8 : CPU_RESET_BIT( m_RegisterBC.hi, 7  ) ; break ;
        case 0xB9 : CPU_RESET_BIT( m_RegisterBC.lo, 7  ) ; break ;
        case 0xBA : CPU_RESET_BIT( m_RegisterDE.hi, 7  ) ; break ;
        case 0xBB : CPU_RESET_BIT( m_RegisterDE.lo, 7  ) ; break ;
        case 0xBC : CPU_RESET_BIT( m_RegisterHL.hi, 7  ) ; break ;
        case 0xBD : CPU_RESET_BIT( m_RegisterHL.lo, 7  ) ; break ;
        case 0xBE : CPU_RESET_BIT_MEMORY( m_RegisterHL.reg, 7 ) ; break ;
        case 0xBF : CPU_RESET_BIT( m_RegisterAF.hi, 7 ) ; break ;


        // set bit
        case 0xC0 : CPU_SET_BIT( m_RegisterBC.hi, 0 ) ; break ;
        case 0xC1 : CPU_SET_BIT( m_RegisterBC.lo, 0 ) ; break ;
        case 0xC2 : CPU_SET_BIT( m_RegisterDE.hi, 0 ) ; break ;
        case 0xC3 : CPU_SET_BIT( m_RegisterDE.lo, 0 ) ; break ;
        case 0xC4 : CPU_SET_BIT( m_RegisterHL.hi, 0 ) ; break ;
        case 0xC5 : CPU_SET_BIT( m_RegisterHL.lo, 0 ) ; break ;
        case 0xC6 : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 0 ) ; break ;
        case 0xC7 : CPU_SET_BIT( m_RegisterAF.hi, 0 ) ; break ;
        case 0xC8 : CPU_SET_BIT( m_RegisterBC.hi, 1  ) ; break ;
        case 0xC9 : CPU_SET_BIT( m_RegisterBC.lo, 1 ) ; break ;
        case 0xCA : CPU_SET_BIT( m_RegisterDE.hi, 1 ) ; break ;
        case 0xCB : CPU_SET_BIT( m_RegisterDE.lo, 1 ) ; break ;
        case 0xCC : CPU_SET_BIT( m_RegisterHL.hi, 1 ) ; break ;
        case 0xCD : CPU_SET_BIT( m_RegisterHL.lo, 1 ) ; break ;
        case 0xCE : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 1 ) ; break ;
        case 0xCF : CPU_SET_BIT( m_RegisterAF.hi, 1  ) ; break ;
        case 0xD0 : CPU_SET_BIT( m_RegisterBC.hi, 2  ) ; break ;
        case 0xD1 : CPU_SET_BIT( m_RegisterBC.lo, 2  ) ; break ;
        case 0xD2 : CPU_SET_BIT( m_RegisterDE.hi, 2  ) ; break ;
        case 0xD3 : CPU_SET_BIT( m_RegisterDE.lo, 2  ) ; break ;
        case 0xD4 : CPU_SET_BIT( m_RegisterHL.hi, 2  ) ; break ;
        case 0xD5 : CPU_SET_BIT( m_RegisterHL.lo, 2  ) ; break ;
        case 0xD6 : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 2 ) ; break ;
        case 0xD7 : CPU_SET_BIT( m_RegisterAF.hi, 2  ) ; break ;
        case 0xD8 : CPU_SET_BIT( m_RegisterBC.hi, 3  ) ; break ;
        case 0xD9 : CPU_SET_BIT( m_RegisterBC.lo, 3  ) ; break ;
        case 0xDA : CPU_SET_BIT( m_RegisterDE.hi, 3  ) ; break ;
        case 0xDB : CPU_SET_BIT( m_RegisterDE.lo, 3  ) ; break ;
        case 0xDC : CPU_SET_BIT( m_RegisterHL.hi, 3  ) ; break ;
        case 0xDD : CPU_SET_BIT( m_RegisterHL.lo, 3  ) ; break ;
        case 0xDE : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 3  ) ; break ;
        case 0xDF : CPU_SET_BIT( m_RegisterAF.hi, 3  ) ; break ;
        case 0xE0 : CPU_SET_BIT( m_RegisterBC.hi, 4  ) ; break ;
        case 0xE1 : CPU_SET_BIT( m_RegisterBC.lo, 4  ) ; break ;
        case 0xE2 : CPU_SET_BIT( m_RegisterDE.hi, 4  ) ; break ;
        case 0xE3 : CPU_SET_BIT( m_RegisterDE.lo, 4  ) ; break ;
        case 0xE4 : CPU_SET_BIT( m_RegisterHL.hi, 4  ) ; break ;
        case 0xE5 : CPU_SET_BIT( m_RegisterHL.lo, 4  ) ; break ;
        case 0xE6 : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 4) ; break ;
        case 0xE7 : CPU_SET_BIT( m_RegisterAF.hi, 4 ) ; break ;
        case 0xE8 : CPU_SET_BIT( m_RegisterBC.hi, 5 ) ; break ;
        case 0xE9 : CPU_SET_BIT( m_RegisterBC.lo, 5 ) ; break ;
        case 0xEA : CPU_SET_BIT( m_RegisterDE.hi, 5 ) ; break ;
        case 0xEB : CPU_SET_BIT( m_RegisterDE.lo, 5 ) ; break ;
        case 0xEC : CPU_SET_BIT( m_RegisterHL.hi, 5 ) ; break ;
        case 0xED : CPU_SET_BIT( m_RegisterHL.lo, 5 ) ; break ;
        case 0xEE : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 5 ) ; break ;
        case 0xEF : CPU_SET_BIT( m_RegisterAF.hi, 5  ) ; break ;
        case 0xF0 : CPU_SET_BIT( m_RegisterBC.hi, 6  ) ; break ;
        case 0xF1 : CPU_SET_BIT( m_RegisterBC.lo, 6  ) ; break ;
        case 0xF2 : CPU_SET_BIT( m_RegisterDE.hi, 6  ) ; break ;
        case 0xF3 : CPU_SET_BIT( m_RegisterDE.lo, 6  ) ; break ;
        case 0xF4 : CPU_SET_BIT( m_RegisterHL.hi, 6  ) ; break ;
        case 0xF5 : CPU_SET_BIT( m_RegisterHL.lo, 6  ) ; break ;
        case 0xF6 : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 6 ) ; break ;
        case 0xF7 : CPU_SET_BIT( m_RegisterAF.hi, 6 ) ; break ;
        case 0xF8 : CPU_SET_BIT( m_RegisterBC.hi, 7  ) ; break ;
        case 0xF9 : CPU_SET_BIT( m_RegisterBC.lo, 7  ) ; break ;
        case 0xFA : CPU_SET_BIT( m_RegisterDE.hi, 7  ) ; break ;
        case 0xFB : CPU_SET_BIT( m_RegisterDE.lo, 7  ) ; break ;
        case 0xFC : CPU_SET_BIT( m_RegisterHL.hi, 7  ) ; break ;
        case 0xFD : CPU_SET_BIT( m_RegisterHL.lo, 7  ) ; break ;
        case 0xFE : CPU_SET_BIT_MEMORY( m_RegisterHL.reg, 7 ) ; break ;
        case 0xFF : CPU_SET_BIT( m_RegisterAF.hi, 7 ) ; break ;


        default:
        {
            char buffer[256];
            sprintf(buffer, "Unhandled Extended Opcode %x", opcode) ;
            LogMessage::GetSingleton()->DoLogMessage(buffer,true) ;
            assert(false) ;
        } break;
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

 */

/*
    Helper functions file:
    #include "Config.h"
#include "Emulator.h"

//////////////////////////////////////////////////////////////////////////////////

// put 1 byte immediate data into reg
void Emulator::CPU_8BIT_LOAD( BYTE& reg )
{
    m_CyclesThisUpdate += 8 ;
    BYTE n = ReadMemory(m_ProgramCounter) ;
    m_ProgramCounter++ ;
    reg = n ;
}

//////////////////////////////////////////////////////////////////////////////////

// put 2 byte immediate data into reg
void Emulator::CPU_16BIT_LOAD( WORD& reg )
{
    m_CyclesThisUpdate += 12 ;
    WORD n = ReadWord() ;
    m_ProgramCounter+=2 ;
    reg = n ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_REG_LOAD(BYTE& reg, BYTE load, int cycles)
{
    m_CyclesThisUpdate += cycles ;
    reg = load ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_REG_LOAD_ROM(BYTE& reg, WORD address)
{
    m_CyclesThisUpdate+=8 ;
    reg = ReadMemory(address) ;
}

//////////////////////////////////////////////////////////////////////////////////

// apparently no flags affected
void Emulator::CPU_16BIT_DEC(WORD& word, int cycles)
{
    m_CyclesThisUpdate+=cycles ;
    word--;
}

//////////////////////////////////////////////////////////////////////////////////

// apparently no flags affected
void Emulator::CPU_16BIT_INC(WORD& word, int cycles)
{
    m_CyclesThisUpdate+=cycles;
    word++;
}

//////////////////////////////////////////////////////////////////////////////////

// add to reg. Can be immediate data, and can also add the carry flag to the result
void Emulator::CPU_8BIT_ADD(BYTE& reg, BYTE toAdd, int cycles, bool useImmediate, bool addCarry)
{
    m_CyclesThisUpdate+=cycles ;
    BYTE before = reg ;
    BYTE adding = 0 ;

    // are we adding immediate data or the second param?
    if (useImmediate)
    {
        BYTE n = ReadMemory(m_ProgramCounter) ;
        m_ProgramCounter++ ;
        adding = n ;
    }
    else
    {
        adding = toAdd ;
    }

    // are we also adding the carry flag?
    if (addCarry)
    {
        if (TestBit(m_RegisterAF.lo, FLAG_C))
            adding++ ;
    }

    reg+=adding ;

    // set the flags
    m_RegisterAF.lo = 0 ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WORD htest = (before & 0xF) ;
    htest += (adding & 0xF) ;

    if (htest > 0xF)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;

    if ((before + adding) > 0xFF)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

//	_asm int 3; // IM UNSURE IF the flags for FLAG C and FLAG H are correct... Need to check
}

//////////////////////////////////////////////////////////////////////////////////

// subtracts away from reg, can also subtract the carry flag too
void Emulator::CPU_8BIT_SUB(BYTE& reg, BYTE subtracting, int cycles, bool useImmediate, bool subCarry)
{
    m_CyclesThisUpdate += cycles ;
    BYTE before = reg ;
    BYTE toSubtract = 0 ;

    if (useImmediate)
    {
        BYTE n = ReadMemory(m_ProgramCounter) ;
        m_ProgramCounter++ ;
        toSubtract = n ;
    }
    else
    {
        toSubtract = subtracting ;
    }

    if (subCarry)
    {
        if (TestBit(m_RegisterAF.lo, FLAG_C))
            toSubtract++ ;
    }

    reg -= toSubtract ;

    m_RegisterAF.lo = 0 ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_N) ;

    // set if no borrow
    if (before < toSubtract)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    SIGNED_WORD htest = (before & 0xF) ;
    htest -= (toSubtract & 0xF) ;

    if (htest < 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;

}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_AND(BYTE& reg, BYTE toAnd, int cycles, bool useImmediate)
{
    m_CyclesThisUpdate+=cycles ;
    BYTE myand = 0 ;

    if (useImmediate)
    {
        BYTE n = ReadMemory(m_ProgramCounter) ;
        m_ProgramCounter++ ;
        myand = n ;
    }
    else
    {
        myand = toAnd ;
    }

    reg &= myand ;

    m_RegisterAF.lo = 0 ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_OR(BYTE& reg, BYTE toOr, int cycles, bool useImmediate)
{
    m_CyclesThisUpdate+=cycles ;
    BYTE myor = 0 ;

    if (useImmediate)
    {
        BYTE n = ReadMemory(m_ProgramCounter) ;
        m_ProgramCounter++ ;
        myor = n ;
    }
    else
    {
        myor = toOr ;
    }

    reg |= myor ;

    m_RegisterAF.lo = 0 ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_XOR(BYTE& reg, BYTE toXOr, int cycles, bool useImmediate)
{
    m_CyclesThisUpdate+=cycles ;
    BYTE myxor = 0 ;

    if (useImmediate)
    {
        BYTE n = ReadMemory(m_ProgramCounter) ;
        m_ProgramCounter++ ;
        myxor = n ;
    }
    else
    {
        myxor = toXOr ;
    }

    reg ^= myxor ;

    m_RegisterAF.lo = 0 ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

// this does not affect any registers, hence why im not passing a reference

void Emulator::CPU_8BIT_COMPARE(BYTE reg, BYTE subtracting, int cycles, bool useImmediate)
{
    m_CyclesThisUpdate += cycles ;
    BYTE before = reg ;
    BYTE toSubtract = 0 ;

    if (useImmediate)
    {
        BYTE n = ReadMemory(m_ProgramCounter) ;
        m_ProgramCounter++ ;
        toSubtract = n ;
    }
    else
    {
        toSubtract = subtracting ;
    }

    reg -= toSubtract ;

    m_RegisterAF.lo = 0 ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_N) ;

    // set if no borrow
    if (before < toSubtract)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;


    SIGNED_WORD htest = before & 0xF ;
    htest -= (toSubtract & 0xF) ;

    if (htest < 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;

}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_INC(BYTE& reg, int cycles)
{
    // WHEN EDITING THIS FUNCTION DONT FORGET TO MAKE THE SAME CHANGES TO CPU_8BIT_MEMORY_INC

    m_CyclesThisUpdate+= cycles ;

    BYTE before = reg ;

    reg++ ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N) ;

    if ((before & 0xF) == 0xF)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_MEMORY_INC(WORD address, int cycles)
{
    // WHEN EDITING THIS FUNCTION DONT FORGET TO MAKE THE SAME CHANGES TO CPU_8BIT_INC

    m_CyclesThisUpdate+= cycles ;

    BYTE before = ReadMemory( address ) ;
    WriteByte(address, (before+1)) ;
    BYTE now =  before+1 ;

    if (now == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N) ;

    if ((before & 0xF) == 0xF)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_DEC(BYTE& reg, int cycles)
{
    // WHEN EDITING THIS FUNCTION DONT FORGET TO MAKE THE SAME CHANGES TO CPU_8BIT_MEMORY_DEC

    m_CyclesThisUpdate+=cycles ;
    BYTE before = reg ;

    reg-- ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_N) ;

    if ((before & 0x0F) == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H) ;

}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_8BIT_MEMORY_DEC(WORD address, int cycles)
{
    // WHEN EDITING THIS FUNCTION DONT FORGET TO MAKE THE SAME CHANGES TO CPU_8BIT_DEC

    m_CyclesThisUpdate+=cycles ;
    BYTE before = ReadMemory(address) ;
    WriteByte(address, (before-1)) ;
    BYTE now = before-1 ;

    if (now == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_N) ;

    if ((before & 0x0F) == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_16BIT_ADD(WORD& reg, WORD toAdd, int cycles)
{
    m_CyclesThisUpdate += cycles ;
    WORD before = reg ;

    reg += toAdd ;

    m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N) ;

    if ((before + toAdd) > 0xFFFF)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_C) ;


    if (( (before & 0xFF00) & 0xF) + ((toAdd >> 8) & 0xF))
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;
    else
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_H) ;

//	_asm int 3; // not sure about flag h
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_JUMP(bool useCondition, int flag, bool condition)
{
    m_CyclesThisUpdate += 12 ;

    WORD nn = ReadWord( ) ;
    m_ProgramCounter += 2 ;

    if (!useCondition)
    {
        m_ProgramCounter = nn ;
        return ;
    }

    if (TestBit(m_RegisterAF.lo, flag) == condition)
    {
        m_ProgramCounter = nn ;
    }

}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_JUMP_IMMEDIATE(bool useCondition, int flag, bool condition)
{
    m_CyclesThisUpdate += 8 ;

    SIGNED_BYTE n = (SIGNED_BYTE)ReadMemory(m_ProgramCounter) ;

    if (!useCondition)
    {
        m_ProgramCounter += n;
    }
    else if (TestBit(m_RegisterAF.lo, flag) == condition)
    {
        m_ProgramCounter += n ;
    }

    m_ProgramCounter++ ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_CALL(bool useCondition, int flag, bool condition)
{
    m_CyclesThisUpdate+=12 ;
    WORD nn = ReadWord( ) ;
    m_ProgramCounter += 2;

    if (!useCondition)
    {
        PushWordOntoStack(m_ProgramCounter) ;
        m_ProgramCounter = nn ;
        return ;
    }

    if (TestBit(m_RegisterAF.lo, flag)==condition)
    {
        PushWordOntoStack(m_ProgramCounter) ;
        m_ProgramCounter = nn ;
    }
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_RETURN(bool useCondition, int flag, bool condition)
{
    m_CyclesThisUpdate += 8 ;
    if (!useCondition)
    {
        m_ProgramCounter = PopWordOffStack( ) ;
        return ;
    }

    if (TestBit(m_RegisterAF.lo, flag) == condition)
    {
        m_ProgramCounter = PopWordOffStack( ) ;
    }
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SWAP_NIBBLES(BYTE& reg)
{
    m_CyclesThisUpdate += 8 ;

    m_RegisterAF.lo = 0 ;

    reg = (((reg & 0xF0) >> 4) | ((reg & 0x0F) << 4));

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SWAP_NIB_MEM
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SWAP_NIB_MEM(WORD address)
{
    m_CyclesThisUpdate += 16 ;

    m_RegisterAF.lo = 0 ;

    BYTE mem = ReadMemory(address) ;
    mem = (((mem & 0xF0) >> 4) | ((mem & 0x0F) << 4));

    WriteByte(address,mem) ;

    if (mem == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;


    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SWAP_NIBBLES


}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_RESTARTS(BYTE n)
{
    PushWordOntoStack(m_ProgramCounter) ;
    m_CyclesThisUpdate += 32 ;
    m_ProgramCounter = n ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SHIFT_LEFT_CARRY(BYTE& reg)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SHIFT_LEFT_CARRY_MEMORY
    m_CyclesThisUpdate += 8 ;
    m_RegisterAF.lo = 0 ;
    if (TestBit(reg,7))
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    reg = reg << 1 ;
    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SHIFT_LEFT_CARRY_MEMORY(WORD address)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SHIFT_LEFT_CARRY
    m_CyclesThisUpdate += 16 ;
    BYTE before = ReadMemory(address) ;

    m_RegisterAF.lo = 0 ;
    if (TestBit(before,7))
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    before = before << 1 ;
    if (before == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, before) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_RESET_BIT(BYTE& reg, int bit)
{
    // WHEN EDITING THIS ALSO EDIT CPU_RESET_BIT_MEMORY
    reg = BitReset(reg, bit) ;
    m_CyclesThisUpdate += 8 ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_RESET_BIT_MEMORY(WORD address, int bit)
{
    // WHEN EDITING THIS ALSO EDIT CPU_RESET_BIT
    BYTE mem = ReadMemory(address) ;
    mem = BitReset(mem, bit) ;
    WriteByte(address, mem) ;
    m_CyclesThisUpdate += 16 ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_TEST_BIT(BYTE reg, int bit, int cycles)
{
    if (TestBit(reg, bit))
        m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_Z) ;
    else
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    m_RegisterAF.lo = BitReset(m_RegisterAF.lo, FLAG_N) ;
    m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_H) ;

    m_CyclesThisUpdate += cycles ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SET_BIT(BYTE& reg, int bit)
{
    // WHEN EDITING THIS ALSO EDIT CPU_SET_BIT_MEMORY
    reg = BitSet(reg, bit) ;
    m_CyclesThisUpdate += 8 ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SET_BIT_MEMORY(WORD address, int bit)
{
    // WHEN EDITING THIS ALSO EDIT CPU_SET_BIT
    BYTE mem = ReadMemory(address) ;
    mem = BitSet(mem, bit) ;
    WriteByte(address, mem) ;
    m_CyclesThisUpdate += 16 ;
}

//////////////////////////////////////////////////////////////////////////////////

// STOLEN
void Emulator::CPU_DAA( )
{
    m_CyclesThisUpdate += 4 ;

    if(TestBit(m_RegisterAF.lo, FLAG_N))
    {
        if((m_RegisterAF.hi &0x0F ) >0x09 || m_RegisterAF.lo &0x20 )
        {
            m_RegisterAF.hi -=0x06; //Half borrow: (0-1) = (0xF-0x6) = 9
            if((m_RegisterAF.hi&0xF0)==0xF0) m_RegisterAF.lo|=0x10; else m_RegisterAF.lo&=~0x10;
        }

        if((m_RegisterAF.hi&0xF0)>0x90 || m_RegisterAF.lo&0x10) m_RegisterAF.hi-=0x60;
    }
    else
    {
        if((m_RegisterAF.hi&0x0F)>9 || m_RegisterAF.lo&0x20)
        {
            m_RegisterAF.hi+=0x06; //Half carry: (9+1) = (0xA+0x6) = 10
            if((m_RegisterAF.hi&0xF0)==0) m_RegisterAF.lo|=0x10; else m_RegisterAF.lo&=~0x10;
        }

        if((m_RegisterAF.hi&0xF0)>0x90 || m_RegisterAF.lo&0x10) m_RegisterAF.hi+=0x60;
    }

    if(m_RegisterAF.hi==0) m_RegisterAF.lo|=0x80; else m_RegisterAF.lo&=~0x80;
}

//////////////////////////////////////////////////////////////////////////////////

// rotate right through carry
void Emulator::CPU_RR(BYTE& reg)
{
    // WHEN EDITING THIS ALSO EDIT CPU_RR_MEMORY
    m_CyclesThisUpdate += 8 ;

    bool isCarrySet = TestBit(m_RegisterAF.lo, FLAG_C) ;
    bool isLSBSet = TestBit(reg, 0) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1 ;

    if (isLSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (isCarrySet)
        reg = BitSet(reg, 7) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

// rotate right through carry
void Emulator::CPU_RR_MEMORY(WORD address)
{
    // WHEN EDITING THIS ALSO EDIT CPU_RR

    m_CyclesThisUpdate += 16 ;

    BYTE reg = ReadMemory(address) ;

    bool isCarrySet = TestBit(m_RegisterAF.lo, FLAG_C) ;
    bool isLSBSet = TestBit(reg, 0) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1 ;

    if (isLSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (isCarrySet)
        reg = BitSet(reg, 7) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;
}

//////////////////////////////////////////////////////////////////////////////////

// rotate left
void Emulator::CPU_RLC(BYTE& reg)
{
    //WHEN EDITING THIS FUNCTION ALSO EDIT CPU_RLC_MEMORY

    m_CyclesThisUpdate += 8 ;

    bool isMSBSet = TestBit(reg, 7) ;

    m_RegisterAF.lo = 0 ;

    reg <<= 1;

    if (isMSBSet)
    {
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;
        reg = BitSet(reg,0) ;
    }

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

}

//////////////////////////////////////////////////////////////////////////////////

// rotate left
void Emulator::CPU_RLC_MEMORY(WORD address)
{
    //WHEN EDITING THIS FUNCTION ALSO EDIT CPU_RLC

    m_CyclesThisUpdate += 16 ;

    BYTE reg = ReadMemory(address) ;

    bool isMSBSet = TestBit(reg, 7) ;

    m_RegisterAF.lo = 0 ;

    reg <<= 1;

    if (isMSBSet)
    {
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;
        reg = BitSet(reg,0) ;
    }

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;

}

//////////////////////////////////////////////////////////////////////////////////

// rotate right
void Emulator::CPU_RRC(BYTE& reg)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_RRC_MEMORY

    m_CyclesThisUpdate += 8 ;

    bool isLSBSet = TestBit(reg, 0) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1;

    if (isLSBSet)
    {
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;
        reg = BitSet(reg,7) ;
    }

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

// rotate right
void Emulator::CPU_RRC_MEMORY(WORD address)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_RRC

    m_CyclesThisUpdate += 16 ;

    BYTE reg = ReadMemory(address) ;

    bool isLSBSet = TestBit(reg, 0) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1;

    if (isLSBSet)
    {
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;
        reg = BitSet(reg,7) ;
    }

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;
}

//////////////////////////////////////////////////////////////////////////////////

// shift left arithmetically (basically bit 0 gets set to 0) (bit 7 goes into carry)
void Emulator::CPU_SLA(BYTE& reg)
{
    // WHEN EDITING THIS ALSO EDIT CPU_SLA_MEMORY

    m_CyclesThisUpdate += 8 ;

    bool isMSBSet = TestBit(reg, 7);

    reg <<= 1;

    m_RegisterAF.lo = 0 ;

    if (isMSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

void Emulator::CPU_SLA_MEMORY(WORD address)
{
    // WHEN EDITING THIS ALSO EDIT CPU_SLA_MEMORY

    m_CyclesThisUpdate += 16 ;

    BYTE reg = ReadMemory(address) ;

    bool isMSBSet = TestBit(reg, 7);

    reg <<= 1;

    m_RegisterAF.lo = 0 ;

    if (isMSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;
}
//////////////////////////////////////////////////////////////////////////////////

// shift right. LSB into carry. bit 7 doesn't change
void Emulator::CPU_SRA(BYTE& reg)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SRA_MEMORY

    m_CyclesThisUpdate += 8 ;

    bool isLSBSet = TestBit(reg,0) ;
    bool isMSBSet = TestBit(reg,7) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1;

    if (isMSBSet)
        reg = BitSet(reg,7) ;
    if (isLSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

// shift right. LSB into carry. bit 7 doesn't change
void Emulator::CPU_SRA_MEMORY(WORD address)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SRA

    m_CyclesThisUpdate += 16 ;

    BYTE reg = ReadMemory(address) ;

    bool isLSBSet = TestBit(reg,0) ;
    bool isMSBSet = TestBit(reg,7) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1;

    if (isMSBSet)
        reg = BitSet(reg,7) ;
    if (isLSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;
}

//////////////////////////////////////////////////////////////////////////////////

// shift right. bit 0 into carry
void Emulator::CPU_SRL(BYTE& reg)
{
    //WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SRL_MEMORY

    m_CyclesThisUpdate += 8 ;

    bool isLSBSet = TestBit(reg,0) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1;

    if (isLSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

}

//////////////////////////////////////////////////////////////////////////////////

// shift right. bit 0 into carry
void Emulator::CPU_SRL_MEMORY(WORD address)
{
    //WHEN EDITING THIS FUNCTION ALSO EDIT CPU_SRL

    m_CyclesThisUpdate += 8 ;

    BYTE reg = ReadMemory(address) ;

    bool isLSBSet = TestBit(reg,0) ;

    m_RegisterAF.lo = 0 ;

    reg >>= 1;

    if (isLSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;

}


// rotate left through carry flag
void Emulator::CPU_RL(BYTE& reg)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_RL_MEMORY
    m_CyclesThisUpdate += 8 ;

    bool isCarrySet = TestBit(m_RegisterAF.lo, FLAG_C) ;
    bool isMSBSet = TestBit(reg, 7) ;

    m_RegisterAF.lo = 0 ;

    reg <<= 1 ;

    if (isMSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (isCarrySet)
        reg = BitSet(reg, 0) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;
}

//////////////////////////////////////////////////////////////////////////////////

// rotate left through carry flag
void Emulator::CPU_RL_MEMORY(WORD address)
{
    // WHEN EDITING THIS FUNCTION ALSO EDIT CPU_RL

    m_CyclesThisUpdate += 16 ;
    BYTE reg = ReadMemory(address) ;

    bool isCarrySet = TestBit(m_RegisterAF.lo, FLAG_C) ;
    bool isMSBSet = TestBit(reg, 7) ;

    m_RegisterAF.lo = 0 ;

    reg <<= 1 ;

    if (isMSBSet)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_C) ;

    if (isCarrySet)
        reg = BitSet(reg, 0) ;

    if (reg == 0)
        m_RegisterAF.lo = BitSet(m_RegisterAF.lo, FLAG_Z) ;

    WriteByte(address, reg) ;
}

//////////////////////////////////////////////////////////////////////////////////

 */

// macro for loading 8-bit values into registers
macro_rules! cpu_8bit_load {
    ($emulator: ident, $register:ident) => {{
        $emulator.cpu.$register = $emulator.read_byte();
        8
    }};
}

macro_rules! cpu_16bit_load {
    ($emulator: ident, $hi: ident, $lo: ident, $word: expr) => {{
        $emulator.cpu.$hi = $word.hi();
        $emulator.cpu.$lo = $word.lo();
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
    ($emulator: ident, $reg: ident) => {{
        let isMSBSet = $emulator.cpu.$reg.test_bit(7);
        $emulator.cpu.$reg <<= 1;
        if isMSBSet {
            $emulator.cpu.$reg.set_bit(0);
        }
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, isMSBSet);
        $emulator
            .cpu
            .f
            .toggle_bit(FLAG_ZERO, $emulator.cpu.$reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }};
}

macro_rules! rotate_right {
    ($emulator: ident, $reg: ident) => {{
        let isLSBSet = $emulator.cpu.$reg.test_bit(0);
        $emulator.cpu.$reg >>= 1;
        if isLSBSet {
            $emulator.cpu.$reg.set_bit(7);
        }
        $emulator.cpu.f.toggle_bit(FLAG_CARRY, isLSBSet);
        $emulator
            .cpu
            .f
            .toggle_bit(FLAG_ZERO, $emulator.cpu.$reg == 0);
        $emulator.cpu.f.reset_bit(FLAG_SUBTRACT);
        $emulator.cpu.f.reset_bit(FLAG_HALF_CARRY);
        8
    }};
}

macro_rules! rotate_left_carry {
    ($emulator: ident, $reg: ident) => {{
        let isCarrySet = $emulator.cpu.f.test_bit(FLAG_CARRY);
        rotate_left!($emulator, $reg);
        $emulator.cpu.$reg.toggle_bit(0, isCarrySet);
        8
    }};
}

macro_rules! rotate_right_carry {
    ($emulator: ident, $reg: ident) => {{
        let isCarrySet = $emulator.cpu.f.test_bit(FLAG_CARRY);
        rotate_right!($emulator, $reg);
        $emulator.cpu.$reg.toggle_bit(7, isCarrySet);
        8
    }};
}

impl Emulator {
    fn read_byte(&mut self) -> u8 {
        let result = self.read_memory(self.cpu.pc);
        self.cpu.pc += 1;
        result
    }

    fn read_word(&mut self) -> u16 {
        u16::from_bytes(self.read_byte(), self.read_byte())
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
            0xF1 => cpu_16bit_load!(self, a, f, self.pop_stack()),
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

            0x07 => rotate_left!(self, a),
            0x0F => rotate_right!(self, a),
            0x17 => rotate_left_carry!(self, a),
            0x1F => rotate_right_carry!(self, a),

            0xD9 => {
                self.cpu.pc = self.pop_stack();
                self.interrupts_enabled = true;
                8
            }

            0x08 => {
                let address = self.read_word();
                self.write_memory(address, self.cpu.sp.lo());
                self.write_memory(address.wrapping_add(1), self.cpu.sp.hi());
                self.cpu.pc = self.cpu.pc.wrapping_add(2);
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

            0xF3 => {
                todo!();
                4
            }

            0xFB => {
                todo!();
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
                todo!();
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
                let offset = self.read_byte() as i16;
                let (result, carry) = self.cpu.sp.overflowing_add_signed(offset);
                self.cpu.f.reset_bit(FLAG_ZERO);
                self.cpu.f.reset_bit(FLAG_SUBTRACT);
                self.cpu.f.toggle_bit(
                    FLAG_HALF_CARRY,
                    (self.cpu.sp & 0x0f).wrapping_add_signed(offset & 0x0f) > 0x0f,
                );
                self.cpu.f.toggle_bit(FLAG_CARRY, carry);
                self.cpu.h = result.hi();
                self.cpu.l = result.lo();
                12
            }

            0x10 => {
                self.cpu.pc += 1;
                4
            }

            _ => panic!("Unknown opcode: {:02X}", opcode),
        }
    }

    fn execute_extended(&mut self) -> u32 {
        todo!();
    }

    fn jump(&mut self, flag: u8, use_condition: bool, condition: bool) -> u32 {
        let address = self.read_word();
        if !use_condition || self.cpu.f.test_bit(flag) == condition {
            self.cpu.pc = address;
        }
        12
    }

    fn jump_immediate(&mut self, flag: u8, use_condition: bool, condition: bool) -> u32 {
        let offset = self.read_byte() as i16;
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
        let result = self.cpu.a & value;
        self.cpu.f.toggle_bit(FLAG_ZERO, result == 0);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.set_bit(FLAG_HALF_CARRY);
        self.cpu.f.reset_bit(FLAG_CARRY);
        self.cpu.a = result;
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
        let (result, carry) = u16::from_bytes(self.cpu.h, self.cpu.l).overflowing_add(value);
        self.cpu.f.reset_bit(FLAG_SUBTRACT);
        self.cpu.f.toggle_bit(
            FLAG_HALF_CARRY,
            (self.cpu.l & 0x0f) + (value & 0x0f) as u8 > 0x0f,
        );
        self.cpu.f.toggle_bit(FLAG_CARRY, carry);
        self.cpu.h = result.hi();
        self.cpu.l = result.lo();
        8
    }
}

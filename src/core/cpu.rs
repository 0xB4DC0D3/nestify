use std::cell::RefCell;
use std::rc::Rc;

use super::bus::Bus;
use super::clock::Clock;
use super::memorymap::MemoryMapType;
use super::registers::Register;
use super::registers::cpu::status::{CpuStatusRegister, CpuStatusRegisterFlags};
use super::memory::Memory;

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

pub struct Instruction<'a> {
    opcode: u8,
    name: &'a str,
    bytes: u8,
    cycles: u8,
    addressing_mode: AddressingMode,
}

impl Instruction<'_> {
    pub const fn new(opcode: u8, name: &'static str, bytes: u8, cycles: u8, addressing_mode: AddressingMode) -> Self {
        Self {
            opcode,
            name,
            bytes,
            cycles,
            addressing_mode
        }
    }
}

static INSTRUCTIONS: [Instruction; 256] = [
        Instruction::new(0x00, "BRK", 1, 7, AddressingMode::Implicit),
        Instruction::new(0x01, "ORA", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0x02, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x03, "SLO", 2, 8, AddressingMode::IndexedIndirect),
        Instruction::new(0x04, "NOP", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x07, "SLO", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x08, "PHP", 1, 3, AddressingMode::Implicit),
        Instruction::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x0A, "ASL", 1, 2, AddressingMode::Accumulator),
        Instruction::new(0x0B, "ANC", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x0C, "NOP", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x0F, "SLO", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x10, "BPL", 2, 2, AddressingMode::Relative),
        Instruction::new(0x11, "ORA", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0x12, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x13, "SLO", 2, 8, AddressingMode::IndirectIndexed),
        Instruction::new(0x14, "NOP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x17, "SLO", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x18, "CLC", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x19, "ORA", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0x1A, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x1B, "SLO", 3, 7, AddressingMode::AbsoluteY),
        Instruction::new(0x1C, "NOP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x1D, "ORA", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x1E, "ASL", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x1F, "SLO", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x20, "JSR", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x21, "AND", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0x22, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x23, "RLA", 2, 8, AddressingMode::IndexedIndirect),
        Instruction::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x27, "RLA", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x28, "PLP", 1, 4, AddressingMode::Implicit),
        Instruction::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x2A, "ROL", 1, 2, AddressingMode::Accumulator),
        Instruction::new(0x2B, "ANC", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x2F, "RLA", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x30, "BMI", 2, 2, AddressingMode::Relative),
        Instruction::new(0x31, "AND", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0x32, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x33, "RLA", 2, 8, AddressingMode::IndirectIndexed),
        Instruction::new(0x34, "NOP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x35, "AND", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x37, "RLA", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x38, "SEC", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x39, "AND", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0x3A, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x3B, "RLA", 3, 7, AddressingMode::AbsoluteY),
        Instruction::new(0x3C, "NOP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x3D, "AND", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x3E, "ROL", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x3F, "RLA", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x40, "RTI", 1, 6, AddressingMode::Implicit),
        Instruction::new(0x41, "EOR", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0x42, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x43, "SRE", 2, 8, AddressingMode::IndexedIndirect),
        Instruction::new(0x44, "NOP", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x47, "SRE", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x48, "PHA", 1, 3, AddressingMode::Implicit),
        Instruction::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x4A, "LSR", 1, 2, AddressingMode::Accumulator),
        Instruction::new(0x4B, "ASR", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x4C, "JMP", 3, 3, AddressingMode::Absolute),
        Instruction::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x4F, "SRE", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x50, "BVC", 2, 2, AddressingMode::Relative),
        Instruction::new(0x51, "EOR", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0x52, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x53, "SRE", 2, 8, AddressingMode::IndirectIndexed),
        Instruction::new(0x54, "NOP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x57, "SRE", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x58, "CLI", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x59, "EOR", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0x5A, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x5B, "SRE", 3, 7, AddressingMode::AbsoluteY),
        Instruction::new(0x5C, "NOP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x5D, "EOR", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x5E, "LSR", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x5F, "SRE", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x60, "RTS", 1, 6, AddressingMode::Implicit),
        Instruction::new(0x61, "ADC", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0x62, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x63, "RRA", 2, 8, AddressingMode::IndexedIndirect),
        Instruction::new(0x64, "NOP", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x67, "RRA", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0x68, "PLA", 1, 4, AddressingMode::Implicit),
        Instruction::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x6A, "ROR", 1, 2, AddressingMode::Accumulator),
        Instruction::new(0x6B, "ARR", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x6C, "JMP", 3, 5, AddressingMode::Indirect),
        Instruction::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x6F, "RRA", 3, 6, AddressingMode::Absolute),
        Instruction::new(0x70, "BVS", 2, 2, AddressingMode::Relative),
        Instruction::new(0x71, "ADC", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0x72, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x73, "RRA", 2, 8, AddressingMode::IndirectIndexed),
        Instruction::new(0x74, "NOP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x77, "RRA", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0x78, "SEI", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x79, "ADC", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0x7A, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x7B, "RRA", 3, 7, AddressingMode::AbsoluteY),
        Instruction::new(0x7C, "NOP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x7D, "ADC", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0x7E, "ROR", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x7F, "RRA", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0x80, "NOP", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x81, "STA", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0x82, "NOP", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x83, "SAX", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x87, "SAX", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0x88, "DEY", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x89, "NOP", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x8A, "TXA", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x8B, "XAA", 2, 2, AddressingMode::Immediate),
        Instruction::new(0x8C, "STY", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x8D, "STA", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x8E, "STX", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x8F, "SAX", 3, 4, AddressingMode::Absolute),
        Instruction::new(0x90, "BCC", 2, 2, AddressingMode::Relative),
        Instruction::new(0x91, "STA", 2, 6, AddressingMode::IndirectIndexed),
        Instruction::new(0x92, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0x93, "AHX", 2, 6, AddressingMode::IndirectIndexed),
        Instruction::new(0x94, "STY", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x95, "STA", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0x96, "STX", 2, 4, AddressingMode::ZeroPageY),
        Instruction::new(0x97, "SAX", 2, 4, AddressingMode::ZeroPageY),
        Instruction::new(0x98, "TYA", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY),
        Instruction::new(0x9A, "TXS", 1, 2, AddressingMode::Implicit),
        Instruction::new(0x9B, "TAS", 3, 5, AddressingMode::AbsoluteY),
        Instruction::new(0x9C, "SHY", 3, 5, AddressingMode::AbsoluteX),
        Instruction::new(0x9D, "STA", 3, 5, AddressingMode::AbsoluteX),
        Instruction::new(0x9E, "SHX", 3, 5, AddressingMode::AbsoluteY),
        Instruction::new(0x9F, "AHX", 3, 5, AddressingMode::AbsoluteY),
        Instruction::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xA1, "LDA", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xA3, "LAX", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xA7, "LAX", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xA8, "TAY", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xAA, "TAX", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xAB, "LAX", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xAF, "LAX", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xB0, "BCS", 2, 2, AddressingMode::Relative),
        Instruction::new(0xB1, "LDA", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0xB2, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0xB3, "LAX", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPageY),
        Instruction::new(0xB7, "LAX", 2, 4, AddressingMode::ZeroPageY),
        Instruction::new(0xB8, "CLV", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xB9, "LDA", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0xBA, "TSX", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xBB, "LAS", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0xBC, "LDY", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0xBD, "LDA", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0xBE, "LDX", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0xBF, "LAX", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xC1, "CMP", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0xC2, "NOP", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xC3, "DCP", 2, 8, AddressingMode::IndexedIndirect),
        Instruction::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0xC7, "DCP", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0xC8, "INY", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xCA, "DEX", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xCB, "AXS", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute),
        Instruction::new(0xCF, "DCP", 3, 6, AddressingMode::Absolute),
        Instruction::new(0xD0, "BNE", 2, 2, AddressingMode::Relative),
        Instruction::new(0xD1, "CMP", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0xD2, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0xD3, "DCP", 2, 8, AddressingMode::IndirectIndexed),
        Instruction::new(0xD4, "NOP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0xD7, "DCP", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0xD8, "CLD", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xD9, "CMP", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0xDA, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xDB, "DCP", 3, 7, AddressingMode::AbsoluteY),
        Instruction::new(0xDC, "NOP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0xDD, "CMP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0xDE, "DEC", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0xDF, "DCP", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xE1, "SBC", 2, 6, AddressingMode::IndexedIndirect),
        Instruction::new(0xE2, "NOP", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xE3, "ISC", 2, 8, AddressingMode::IndexedIndirect),
        Instruction::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage),
        Instruction::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0xE7, "ISC", 2, 5, AddressingMode::ZeroPage),
        Instruction::new(0xE8, "INX", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xEA, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xEB, "SBC", 2, 2, AddressingMode::Immediate),
        Instruction::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xED, "SBC", 3, 4, AddressingMode::Absolute),
        Instruction::new(0xEE, "INC", 3, 6, AddressingMode::Absolute),
        Instruction::new(0xEF, "ISC", 3, 6, AddressingMode::Absolute),
        Instruction::new(0xF0, "BEQ", 2, 2, AddressingMode::Relative),
        Instruction::new(0xF1, "SBC", 2, 5, AddressingMode::IndirectIndexed),
        Instruction::new(0xF2, "KIL", 1, 1, AddressingMode::Implicit),
        Instruction::new(0xF3, "ISC", 2, 8, AddressingMode::IndirectIndexed),
        Instruction::new(0xF4, "NOP", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPageX),
        Instruction::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0xF7, "ISC", 2, 6, AddressingMode::ZeroPageX),
        Instruction::new(0xF8, "SED", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xF9, "SBC", 3, 4, AddressingMode::AbsoluteY),
        Instruction::new(0xFA, "NOP", 1, 2, AddressingMode::Implicit),
        Instruction::new(0xFB, "ISC", 3, 7, AddressingMode::AbsoluteY),
        Instruction::new(0xFC, "NOP", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0xFD, "SBC", 3, 4, AddressingMode::AbsoluteX),
        Instruction::new(0xFE, "INC", 3, 7, AddressingMode::AbsoluteX),
        Instruction::new(0xFF, "ISC", 3, 7, AddressingMode::AbsoluteX),
];

struct InternalState {
    current_instruction: String,
    args_length: u8,
}

pub struct Cpu {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: CpuStatusRegister,
    stack_pointer: u8,
    program_counter: u16,
    internal_state: Option<InternalState>,
    bus: Rc<RefCell<Bus>>,
    clock: Rc<RefCell<Clock>>,
    use_disassembler: bool,
}

impl Cpu {
    pub fn new(bus: &Rc<RefCell<Bus>>, clock: &Rc<RefCell<Clock>>) -> Self {
        Self {
            register_a: 0x00,
            register_x: 0x00,
            register_y: 0x00,
            status: CpuStatusRegister::new(),
            stack_pointer: 0xFD,
            program_counter: 0x8000,
            internal_state: None,
            bus: bus.clone(),
            clock: clock.clone(),
            use_disassembler: false,
        }
    }

    pub fn use_disassembler(&mut self, active: bool) {
        if active {
            self.use_disassembler = true;
        } else {
            self.use_disassembler = false;
        }
    }

    fn push_stack(&mut self, value: u8) {
        self.write(0x0100 + self.stack_pointer as u16, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn push_stack_u16(&mut self, value: u16) {
        let [lo, hi] = value.to_le_bytes();

        self.push_stack(hi);
        self.push_stack(lo);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read(0x0100 + self.stack_pointer as u16)
    }

    fn pop_stack_u16(&mut self) -> u16 {
        let lo = self.pop_stack();
        let hi = self.pop_stack();

        u16::from_le_bytes([lo, hi])
    }

    pub fn reset(&mut self) {
        self.register_a = 0x00;
        self.register_x = 0x00;
        self.register_y = 0x00;
        self.status = CpuStatusRegister::new();
        self.stack_pointer = 0xFD;
        self.program_counter = self.read_u16(0xFFFC);
        self.clock.borrow_mut().tick(7);
    }

    fn is_page_cross(&self, page1: u16, page2: u16) -> bool {
        (page1 & 0xFF00) != (page2 & 0xFF00)
    }

    pub fn get_memory_data(&self, addressing_mode: &AddressingMode) -> Option<(u16, bool)> {
        let mut instruction_info = String::new();
        let result = match addressing_mode {
            AddressingMode::Implicit => {
                None
            },
            AddressingMode::Accumulator => {
                if self.use_disassembler {
                    instruction_info = "A".into();
                }

                None
            },
            AddressingMode::Immediate => {
                let memory_pointer = self.program_counter;

                if self.use_disassembler {
                    instruction_info = format!("#${:02X}", self.read(memory_pointer));
                }

                Some((memory_pointer, false))
            },
            AddressingMode::ZeroPage => {
                let memory_pointer = self.read(self.program_counter) as u16;

                if self.use_disassembler {
                    instruction_info = format!(
                        "${:02X} = {:02X}",
                        memory_pointer, self.read(memory_pointer)
                    );
                }

                Some((memory_pointer, false))
            },
            AddressingMode::ZeroPageX => {
                let pointer = self
                    .read(self.program_counter);

                let memory_pointer = pointer 
                    .wrapping_add(self.register_x);

                let is_page_cross = self.is_page_cross(pointer as u16, memory_pointer as u16);

                if self.use_disassembler {
                    instruction_info = format!(
                        "${:02X},X @ {:02X} = {:02X}",
                        pointer, memory_pointer, self.read(memory_pointer as u16)
                    );
                }

                Some((memory_pointer as u16, is_page_cross))
            },
            AddressingMode::ZeroPageY => {
                let pointer = self
                    .read(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_y);

                let is_page_cross = self.is_page_cross(pointer as u16, memory_pointer as u16);

                if self.use_disassembler {
                    instruction_info = format!(
                        "${:02X},Y @ {:02X} = {:02X}",
                        pointer, memory_pointer, self.read(memory_pointer as u16)
                    );
                }

                Some((memory_pointer as u16, is_page_cross))
            },
            AddressingMode::Relative => {
                let offset = self.read(self.program_counter) as u16;

                if self.use_disassembler {
                    let jump_offset = offset as i8;
                    instruction_info = format!(
                        "${:04X}", 
                        (self.program_counter.wrapping_add(1) as i16)
                            .wrapping_add(jump_offset as i16) as u16
                    );
                }

                Some((offset, false))
            },
            AddressingMode::Absolute => {
                let memory_pointer = self.read_u16(self.program_counter);

                if self.use_disassembler {
                    let value = if let 0x2000..=0x3FFF = memory_pointer {
                        0x00
                    } else {
                        self.read(memory_pointer)
                    };

                    let current_instruction = &self.internal_state
                        .as_ref()
                        .unwrap()
                        .current_instruction;

                    if let "JSR" | "JMP" = current_instruction.as_str() {
                        instruction_info = format!(
                            "${:04X}",
                            memory_pointer
                        );
                    } else {
                        instruction_info = format!(
                            "${:04X} = {:02X}",
                            memory_pointer, value
                        );
                    }
                }

                Some((self.read_u16(self.program_counter), false))
            },
            AddressingMode::AbsoluteX => {
                let pointer = self
                    .read_u16(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_x as u16);

                let is_page_cross = self.is_page_cross(pointer, memory_pointer);

                if self.use_disassembler {
                    instruction_info = format!(
                        "${:04X},X @ {:04X} = {:02X}",
                        pointer, memory_pointer, self.read(memory_pointer)
                    );
                }

                Some((memory_pointer, is_page_cross))
            },
            AddressingMode::AbsoluteY => {
                let pointer = self
                    .read_u16(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_y as u16);

                let is_page_cross = self.is_page_cross(pointer, memory_pointer);

                if self.use_disassembler {
                    instruction_info = format!(
                        "${:04X},Y @ {:04X} = {:02X}",
                        pointer, memory_pointer, self.read(memory_pointer)
                    );
                }

                Some((memory_pointer, is_page_cross))
            },
            AddressingMode::Indirect => {
                let pointer = self.read_u16(self.program_counter);

                // Indirect addressing modes do not handle page boundary crossing at all.
                // When the parameter's low byte is $FF, the effective address wraps
                // around and the CPU fetches high byte from $xx00 instead of $xx00+$0100.
                // E.g. JMP ($01FF) fetches PCL from $01FF and PCH from $0100,
                // and LDA ($FF),Y fetches the base address from $FF and $00.
                // 0x02FF, 0x0200
                let [lo, hi] = pointer.to_le_bytes();
                let hibyte_pointer = u16::from_le_bytes([lo.wrapping_add(1), hi]);
                let memory_pointer = u16::from_le_bytes([
                    self.read(pointer),
                    self.read(hibyte_pointer),
                ]);

                if self.use_disassembler {
                    instruction_info = format!(
                        "(${:04X}) = {:04X}",
                        pointer, memory_pointer
                    );
                }

                Some((memory_pointer, false))
            },
            AddressingMode::IndexedIndirect => {
                let pointer = self
                    .read(self.program_counter)
                    .wrapping_add(self.register_x) as u16 & 0xFF;

                let lo = self.read(pointer);
                let hi = self.read(pointer.wrapping_add(1) & 0xFF);
                let memory_pointer = u16::from_le_bytes([lo, hi]);

                if self.use_disassembler {
                    instruction_info = format!(
                        "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                        pointer.wrapping_sub(self.register_x as u16), pointer, memory_pointer, self.read(memory_pointer)
                    );
                }

                Some((memory_pointer, false))
            },
            AddressingMode::IndirectIndexed => {
                let pointer = self
                    .read(self.program_counter) as u16;

                let lo = self.read(pointer);
                let hi = self.read(pointer.wrapping_add(1) & 0xFF);
                let deref_pointer = u16::from_le_bytes([lo, hi]);

                let memory_pointer = deref_pointer
                    .wrapping_add(self.register_y as u16);

                let is_page_cross = self.is_page_cross(deref_pointer, memory_pointer as u16);

                if self.use_disassembler {
                    instruction_info = format!(
                        "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                        pointer, deref_pointer, memory_pointer, self.read(memory_pointer)
                    );
                }

                Some((memory_pointer, is_page_cross))
            },
        };
        
        if self.use_disassembler {
            let InternalState { 
                current_instruction, 
                args_length
            } = self.internal_state.as_ref().unwrap();

            let hexdump = (0..*args_length + 1).into_iter()
                .map(|offset| {
                    format!("{:02X}", self.read(self.program_counter.wrapping_sub(1).wrapping_add(offset as u16)))
                })
                .collect::<Vec<_>>()
                .join(" ");

            println!(
                "{:<47} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                format!("{:04X}  {:<9} {} {}", self.program_counter.wrapping_sub(1), hexdump, current_instruction, instruction_info),
                self.register_a, self.register_x, self.register_y, self.status.get(), self.stack_pointer
            );
        }

        result
    }

    pub fn set_program_counter(&mut self, address: u16) {
        self.program_counter = address;
    }

    fn execute_adc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ADC instruction!");

        let a = self.register_a as u16;
        let m = self.read(memory_pointer) as u16;
        let c = if self.status.get_flag(CpuStatusRegisterFlags::Carry) { 1u16 } else { 0u16 };
        let result = a.wrapping_add(m).wrapping_add(c);
        let overflow = (a ^ result) & !(a ^ m) & 0x80 == 0x80;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, result > 255);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result as u8 & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result as u8 == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, overflow);
        self.register_a = result as u8;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_and(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for AND instruction!");

        let result = self.register_a & self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_asl(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value << 1;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.register_a = result;
        }
    }

    fn branch(&mut self, flag_active: bool) {
        let (memory_pointer, _) = self.get_memory_data(&AddressingMode::Relative)
            .expect("Invalid Addressing mode for branch instructions!");

        if flag_active {
            self.clock.borrow_mut().tick(1);

            let offset = memory_pointer as i8;
            let next_pc = self.program_counter.wrapping_add(1);
            let jump_pc = (next_pc as i16).wrapping_add(offset as i16) as u16;

            if self.is_page_cross(next_pc, jump_pc) {
                self.clock.borrow_mut().tick(1);
            }

            self.program_counter = jump_pc;
        }
    }

    fn execute_bcc(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Carry));
    }

    fn execute_bcs(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Carry));
    }

    fn execute_beq(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Zero));
    }

    fn execute_bit(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for BIT instruction!");

        let memory_value = self.read(memory_pointer);
        let result = self.register_a & memory_value;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, memory_value & 0x40 == 0x40);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
    }

    fn execute_bmi(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Negative));
    }

    fn execute_bne(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Zero));
    }

    fn execute_bpl(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Negative));
    }

    fn execute_brk(&self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        // Do nothing
    }

    fn execute_bvc(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Overflow));
    }

    fn execute_bvs(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Overflow));
    }

    fn execute_clc(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Carry, false);
    }

    fn execute_cld(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::DecimalMode, false);
    }

    fn execute_cli(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, false);
    }

    fn execute_clv(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
    }

    fn compare(&mut self, addressing_mode: &AddressingMode, register_value: u8) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for CMP/CPX/CPY instructions!");

        let memory_value = self.read(memory_pointer);
        let result = register_value.wrapping_sub(memory_value);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, register_value >= memory_value);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_cmp(&mut self, addressing_mode: &AddressingMode) {
        self.compare(addressing_mode, self.register_a);
    }

    fn execute_cpx(&mut self, addressing_mode: &AddressingMode) {
        self.compare(addressing_mode, self.register_x);
    }

    fn execute_cpy(&mut self, addressing_mode: &AddressingMode) {
        self.compare(addressing_mode, self.register_y);
    }

    fn execute_dec(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for DEC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);
    }

    fn execute_dex(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let result = self.register_x.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_x = result;
    }

    fn execute_dey(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let result = self.register_y.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_y = result;
    }

    fn execute_eor(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for EOR instruction!");

        let result = self.register_a ^ self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_inc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for INC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);
    }

    fn execute_inx(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let result = self.register_x.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_x = result;
    }

    fn execute_iny(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let result = self.register_y.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_y = result;
    }

    fn execute_jmp(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for JMP instruction!");

        self.program_counter = memory_pointer;
    }

    fn execute_jsr(&mut self) {
        let (memory_pointer, _) = self.get_memory_data(&AddressingMode::Absolute).unwrap();

        self.push_stack_u16(self.program_counter.wrapping_add(1));
        self.program_counter = memory_pointer;
    }

    fn execute_lda(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_a = memory_value;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_ldx(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_x = memory_value;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_ldy(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_y = memory_value;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_lsr(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value >> 1;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.register_a = result;
        }
    }

    fn execute_nop(&self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        // Do nothing
    }

    fn execute_ora(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ORA instruction!");
        
        let result = self.register_a | self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_pha(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.push_stack(self.register_a);
    }

    fn execute_php(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        // PHP always pushes the Break (B) flag as a `1' to the stack.
        self.push_stack(self.status.get() | CpuStatusRegisterFlags::Break as u8);
    }

    fn execute_pla(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let value_from_stack = self.pop_stack();

        self.status.set_flag(CpuStatusRegisterFlags::Zero, value_from_stack == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, value_from_stack & 0x80 == 0x80);
        self.register_a = value_from_stack;
    }

    fn execute_plp(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let status = self.pop_stack();

        // If PHP always pushes the Break (B) flag as `1', then we should
        // restore Break (B) flag, when we're pulling out Status register.
        // Also we should set Unused flag (nestest.log have this flag set
        // after PLP)!

        self.status.set(status);
        self.status.set_flag(CpuStatusRegisterFlags::Break, false);
        self.status.set_flag(CpuStatusRegisterFlags::Unused, true);
    }

    fn execute_rol(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let carry_flag = self.status.get_flag(CpuStatusRegisterFlags::Carry);
        let result = if carry_flag {
            value.rotate_left(1) | 0x1
        } else {
            value.rotate_left(1) & !0x1
        };

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
            self.register_a = result;
        }
    }

    fn execute_ror(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let carry_flag = self.status.get_flag(CpuStatusRegisterFlags::Carry);
        let result = if carry_flag {
            value.rotate_right(1) | 0x80
        } else {
            value.rotate_right(1) & !0x80
        };

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
            self.register_a = result;
        }
    }

    fn execute_rti(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);

        let status = self.pop_stack();
        let program_counter = self.pop_stack_u16();

        self.status.set(status);
        self.status.set_flag(CpuStatusRegisterFlags::Unused, true);
        self.program_counter = program_counter;
    }

    fn execute_rts(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.program_counter = self.pop_stack_u16().wrapping_add(1);
    }

    fn execute_sbc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ADC instruction!");

        let a = self.register_a as u16;
        let m = self.read(memory_pointer) as u16 ^ 0xFF;
        let c = if self.status.get_flag(CpuStatusRegisterFlags::Carry) { 1u16 } else { 0u16 };
        let result = a.wrapping_add(m).wrapping_add(c);
        let overflow = (a ^ result) & !(a ^ m) & 0x80 == 0x80;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, result > 255);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result as u8 & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result as u8 == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, overflow);
        self.register_a = result as u8;

        if additional_cycle {
            self.clock.borrow_mut().tick(1);
        }
    }

    fn execute_sec(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Carry, true);
    }

    fn execute_sed(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::DecimalMode, true);
    }

    fn execute_sei(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, true);
    }

    fn execute_sta(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_a);
    }

    fn execute_stx(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_x);
    }

    fn execute_sty(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_y);
    }

    fn execute_tax(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_x = self.register_a;
    }

    fn execute_tay(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_y = self.register_a;
    }

    fn execute_tsx(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.stack_pointer == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.stack_pointer & 0x80 == 0x80);
        self.register_x = self.stack_pointer;
    }

    fn execute_txa(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_x == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_x & 0x80 == 0x80);
        self.register_a = self.register_x;
    }

    fn execute_txs(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.stack_pointer = self.register_x;
    }

    fn execute_tya(&mut self, addressing_mode: &AddressingMode) {
        self.get_memory_data(addressing_mode);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_y == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_y & 0x80 == 0x80);
        self.register_a = self.register_y;
    }

    // TODO: add tests
    fn execute_lax(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LAX (LDA + TAX) instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_a = memory_value;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_x = self.register_a;
    }

    // TODO: add tests
    fn execute_sax(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for SAX instruction!");

        let result = self.register_a & self.register_x;
        self.write(memory_pointer, result);
    }

    // TODO: add tests
    fn execute_dcp(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for DCP instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);
        
        let register_value = self.register_a;
        let memory_value = self.read(memory_pointer);
        let result = register_value.wrapping_sub(memory_value);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, register_value >= memory_value);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
    }

    // TODO: add tests
    fn execute_isc(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ISC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        let a = self.register_a as u16;
        let m = self.read(memory_pointer) as u16 ^ 0xFF;
        let c = if self.status.get_flag(CpuStatusRegisterFlags::Carry) { 1u16 } else { 0u16 };
        let result = a.wrapping_add(m).wrapping_add(c);
        let overflow = (a ^ result) & !(a ^ m) & 0x80 == 0x80;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, result > 255);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result as u8 & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result as u8 == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, overflow);
        self.register_a = result as u8;
    }

    // TODO: add tests
    fn execute_slo(&mut self, addressing_mode: &AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for SLO instruction!");

        let value = self.read(memory_pointer);
        let result = value << 1;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        let result = self.register_a | result;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;
    }

    // TODO: add tests
    fn execute_rla(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for RLA instruction!");

        let value = self.read(memory_pointer);
        let carry_flag = self.status.get_flag(CpuStatusRegisterFlags::Carry);
        let result = if carry_flag {
            value.rotate_left(1) | 0x1
        } else {
            value.rotate_left(1) & !0x1
        };

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        let result = self.register_a & result;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;
    }

    // TODO: add tests
    fn execute_sre(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for SRE instruction!");

        let value = self.read(memory_pointer);
        let result = value >> 1;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        let result = self.register_a ^ result;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;
    }

    // TODO: add tests
    fn execute_rra(&mut self, addressing_mode: &AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ADC instruction!");

        let value = self.read(memory_pointer);
        let carry_flag = self.status.get_flag(CpuStatusRegisterFlags::Carry);
        let result = if carry_flag {
            value.rotate_right(1) | 0x80
        } else {
            value.rotate_right(1) & !0x80
        };

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        let a = self.register_a as u16;
        let m = self.read(memory_pointer) as u16;
        let c = if self.status.get_flag(CpuStatusRegisterFlags::Carry) { 1u16 } else { 0u16 };
        let result = a.wrapping_add(m).wrapping_add(c);
        let overflow = (a ^ result) & !(a ^ m) & 0x80 == 0x80;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, result > 255);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result as u8 & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result as u8 == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, overflow);
        self.register_a = result as u8;
    }

    pub fn fetch(&mut self) {
        let Instruction {
            opcode,
            bytes,
            name,
            cycles,
            addressing_mode
        } = INSTRUCTIONS[self.read(self.program_counter) as usize];

        self.program_counter = self.program_counter.wrapping_add(1);

        let current_program_counter = self.program_counter;

        if let Some(_) = self.bus.borrow_mut().poll_interrupt() {
            // TODO: add interrupt handle
        }

        self.internal_state = Some(InternalState {
            current_instruction: name.to_string(),
            args_length: bytes - 1
        });

        match name {
            "ADC" => self.execute_adc(&addressing_mode),
            "AND" => self.execute_and(&addressing_mode),
            "ASL" => self.execute_asl(&addressing_mode),
            "BCC" => self.execute_bcc(),
            "BCS" => self.execute_bcs(),
            "BEQ" => self.execute_beq(),
            "BIT" => self.execute_bit(&addressing_mode),
            "BMI" => self.execute_bmi(),
            "BNE" => self.execute_bne(),
            "BPL" => self.execute_bpl(),
            "BRK" => self.execute_brk(&addressing_mode),
            "BVC" => self.execute_bvc(),
            "BVS" => self.execute_bvs(),
            "CLC" => self.execute_clc(&addressing_mode),
            "CLD" => self.execute_cld(&addressing_mode),
            "CLV" => self.execute_clv(&addressing_mode),
            "CMP" => self.execute_cmp(&addressing_mode),
            "CPX" => self.execute_cpx(&addressing_mode),
            "CPY" => self.execute_cpy(&addressing_mode),
            "DEC" => self.execute_dec(&addressing_mode),
            "DEX" => self.execute_dex(&addressing_mode),
            "DEY" => self.execute_dey(&addressing_mode),
            "EOR" => self.execute_eor(&addressing_mode),
            "INC" => self.execute_inc(&addressing_mode),
            "INX" => self.execute_inx(&addressing_mode),
            "INY" => self.execute_iny(&addressing_mode),
            "JMP" => self.execute_jmp(&addressing_mode),
            "JSR" => self.execute_jsr(),
            "LDA" => self.execute_lda(&addressing_mode),
            "LDX" => self.execute_ldx(&addressing_mode),
            "LDY" => self.execute_ldy(&addressing_mode),
            "LSR" => self.execute_lsr(&addressing_mode),
            "NOP" => self.execute_nop(&addressing_mode),
            "ORA" => self.execute_ora(&addressing_mode),
            "PHA" => self.execute_pha(&addressing_mode),
            "PHP" => self.execute_php(&addressing_mode),
            "PLA" => self.execute_pla(&addressing_mode),
            "PLP" => self.execute_plp(&addressing_mode),
            "ROL" => self.execute_rol(&addressing_mode),
            "ROR" => self.execute_ror(&addressing_mode),
            "RTI" => self.execute_rti(&addressing_mode),
            "RTS" => self.execute_rts(&addressing_mode),
            "SBC" => self.execute_sbc(&addressing_mode),
            "SEC" => self.execute_sec(&addressing_mode),
            "SED" => self.execute_sed(&addressing_mode),
            "SEI" => self.execute_sei(&addressing_mode),
            "STA" => self.execute_sta(&addressing_mode),
            "STX" => self.execute_stx(&addressing_mode),
            "STY" => self.execute_sty(&addressing_mode),
            "TAX" => self.execute_tax(&addressing_mode),
            "TAY" => self.execute_tay(&addressing_mode),
            "TSX" => self.execute_tsx(&addressing_mode),
            "TXA" => self.execute_txa(&addressing_mode),
            "TXS" => self.execute_txs(&addressing_mode),
            "TYA" => self.execute_tya(&addressing_mode),
            "LAX" => self.execute_lax(&addressing_mode),
            "SAX" => self.execute_sax(&addressing_mode),
            "DCP" => self.execute_dcp(&addressing_mode),
            "ISC" => self.execute_isc(&addressing_mode),
            "SLO" => self.execute_slo(&addressing_mode),
            "RLA" => self.execute_rla(&addressing_mode),
            "SRE" => self.execute_sre(&addressing_mode),
            "RRA" => self.execute_rra(&addressing_mode),
            _ => panic!("Illegal opcode {:#02X} occured!", opcode),
        }

        self.clock.borrow_mut().tick(cycles as usize);

        if current_program_counter == self.program_counter {
            let args_length = (bytes - 1) as u16;

            self.program_counter = self.program_counter.wrapping_add(args_length);
        }
    }
}

impl Memory for Cpu {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                self.bus
                    .borrow_mut()
                    .get_memory_map(MemoryMapType::Cpu)
                    .read(address & 0x7FF)
            },
            0x2000..=0x3FFF => todo!("PPU registers"),
            0x4000..=0x4017 => todo!("PPU OAM DMA, APU"),
            0x4018..=0x401F => panic!("APU and I/O func. test is normally disabled!"),
            0x4020..=0xFFFF => {
                self.bus
                    .borrow_mut()
                    .get_memory_map(MemoryMapType::Cpu)
                    .read(address)
            },
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.bus
                    .borrow_mut()
                    .get_memory_map(MemoryMapType::Cpu)
                    .write(address & 0x7FF, data);
            },
            0x2000..=0x3FFF => todo!("PPU registers"),
            0x4000..=0x4017 => todo!("PPU OAM DMA, APU"),
            0x4018..=0x401F => panic!("APU and I/O func. test is normally disabled!"),
            0x4020..=0xFFFF => {
                self.bus
                    .borrow_mut()
                    .get_memory_map(MemoryMapType::Cpu)
                    .write(address, data);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cartridge::Cartridge;

    use super::*;

    #[test]
    fn test_adc_instruction() {
        let cartridge = Cartridge::empty();
        let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_a = 127;
        cpu.write(0x0000, 0x69);
        cpu.write(0x0001, 0x7F);
        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);

        cpu.execute_adc(&AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 254, "Register A should be 254!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow flag should be set!");

        cpu.register_a = 128;
        cpu.write(0x0000, 0x69);
        cpu.write(0x0001, 0x80);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);

        cpu.execute_adc(&AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 1, "Register A should be 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow flag should be set!");
    }

    #[test]
    fn test_and_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_a = 127;
        cpu.write(0x0000, 0x29);
        cpu.write(0x0001, 0x7E);
        cpu.program_counter = 0x0001;

        cpu.execute_and(&AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 0x7E, "Register A should be 126!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
    }

    #[test]
    fn test_asl_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_a = 0x80;
        cpu.execute_asl(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0x00, "Register A should be 0!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
    }

    #[test]
    fn test_bcc_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_bcc();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BCC with inactive Carry flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_bcc();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BCC with active Carry flag!");
    }

    #[test]
    fn test_bcs_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_bcs();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BCS with inactive Carry flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_bcs();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BCS with active Carry flag!");
    }

    #[test]
    fn test_beq_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, false);
        cpu.execute_beq();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BEQ with inactive Zero flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.execute_beq();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BEQ with active Zero flag!");
    }

    #[test]
    fn test_bit_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0xFF;
        cpu.program_counter = 0x0001;
        cpu.write(0x0000, 0xC0);

        // 0x0001 - lobyte of 0x0000, 0x0002 - hibyte of 0x0000
        cpu.write(0x0001, 0x00);
        cpu.write(0x0002, 0x00);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be set!");

        cpu.write(0x0000, 0x80);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");

        cpu.write(0x0000, 0x40);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");

        cpu.write(0x0000, 0x3F);
        cpu.execute_bit(&AddressingMode::Absolute);
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");
    }

    #[test]
    fn test_bmi_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, false);
        cpu.execute_bmi();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BMI with inactive Negative flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, true);
        cpu.execute_bmi();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BMI with active negative flag!");
    }

    #[test]
    fn test_bne_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, false);
        cpu.execute_bne();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BNE with inactive Zero flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.execute_bne();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BNE with active Zero flag!");
    }

    #[test]
    fn test_bpl_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, false);
        cpu.execute_bpl();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BPL with inactive Negative flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, true);
        cpu.execute_bpl();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BPL with active Negative flag!");
    }

    #[test]
    fn test_bvc_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
        cpu.execute_bvc();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BVC with inactive Overflow flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_bvc();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BVC with active Overflow flag!");
    }

    #[test]
    fn test_bvs_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_bvs();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BVC with active Overflow flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
        cpu.execute_bvs();
        assert_eq!(cpu.program_counter, 0x0001, "CPU PC should be 0x0001 after BVC with inactive Overflow flag!");
    }

    #[test] 
    fn test_clc_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_clc(&AddressingMode::Implicit);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be unset!");
    }

    #[test]
    fn test_cld_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.status.set_flag(CpuStatusRegisterFlags::DecimalMode, true);
        cpu.execute_cld(&AddressingMode::Implicit);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::DecimalMode), "Decimal mode flag should be unset!");
    }

    #[test]
    fn test_cli_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, true);
        cpu.execute_cli(&AddressingMode::Implicit);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::InterruptDisable), "Interrupt disable flag should be unset!");
    }

    #[test]
    fn test_clv_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_clv(&AddressingMode::Implicit);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be unset!");
    }

    #[test]
    fn test_cmp_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_a = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_a = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(&AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_a = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_cpx_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_x = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_x = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(&AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_x = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_cpy_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_y = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_y = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(&AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_y = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(&AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_dec_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.write(0x0001, 129);
        cpu.write(0x0002, 0x1);
        cpu.program_counter = 0x0002;
        cpu.execute_dec(&AddressingMode::ZeroPageX);

        let memory_value = cpu.read(0x0001);
        assert_eq!(memory_value, 128, "Memory value at 0x0001 should be 128!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_dex_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_x = 128;
        cpu.execute_dex(&AddressingMode::Implicit);

        assert_eq!(cpu.register_x, 127, "Register X should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_dey_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_y = 128;
        cpu.execute_dey(&AddressingMode::Implicit);

        assert_eq!(cpu.register_y, 127, "Register Y should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_eor_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_a = 12;
        cpu.write(0x0000, 37);
        cpu.program_counter = 0x0000;
        cpu.execute_eor(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 41, "Register A should contain result of 12 ^ 37!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_inc_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.write(0x0001, 129);
        cpu.write(0x0002, 0x1);
        cpu.program_counter = 0x0002;
        cpu.execute_inc(&AddressingMode::ZeroPageX);

        let memory_value = cpu.read(0x0001);
        assert_eq!(memory_value, 130, "Memory value at 0x0001 should be 130!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_inx_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
        let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);
        cpu.register_x = 128;
        cpu.execute_inx(&AddressingMode::Implicit);

        assert_eq!(cpu.register_x, 129, "Register X should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_iny_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_y = 128;
        cpu.execute_iny(&AddressingMode::Implicit);

        assert_eq!(cpu.register_y, 129, "Register Y should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_jmp_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xFF);
        cpu.write(0x0001, 0xAA);
        cpu.execute_jmp(&AddressingMode::Absolute);

        assert_eq!(cpu.program_counter, 0xAAFF, "Program counter should be 0xAAFF!");
    }

    #[test]
    fn test_jsr_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        let stack_pointer_buf = cpu.stack_pointer;
        cpu.program_counter = 0x0400;
        cpu.write(0x0400, 0xFF);
        cpu.write(0x0401, 0xAA);
        cpu.execute_jsr();

        assert_eq!(cpu.program_counter, 0xAAFF, "Program counter should be 0xAAFF");
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(2), "Invalid stack pointer!");

        let lo = cpu.read(0x0100 + cpu.stack_pointer.wrapping_add(1) as u16);
        let hi = cpu.read(0x0100 + cpu.stack_pointer.wrapping_add(2) as u16);
        assert_eq!(lo, 0x01, "Invalid lobyte of PC in Stack!");
        assert_eq!(hi, 0x04, "Invalid hibyte of PC in Stack!");
    }

    #[test]
    fn test_lda_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_lda(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_lda(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_ldx_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_ldx(&AddressingMode::Immediate);

        assert_eq!(cpu.register_x, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_ldx(&AddressingMode::Immediate);

        assert_eq!(cpu.register_x, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_ldy_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_ldy(&AddressingMode::Immediate);

        assert_eq!(cpu.register_y, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_ldy(&AddressingMode::Immediate);

        assert_eq!(cpu.register_y, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_lsr_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0x81;
        cpu.execute_lsr(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0x40, "Register A should be 0x40!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
    }

    #[test]
    fn test_ora_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0x81;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xFF);
        cpu.execute_ora(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0xFF, "Register A should be 0xFF!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_pha_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        let stack_pointer_buf = cpu.stack_pointer;

        cpu.execute_pha(&AddressingMode::Implicit);
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(1), "Invalid Stack pointer!");

        let register_from_stack = cpu.read(0x0100 + stack_pointer_buf as u16);
        assert_eq!(cpu.register_a, register_from_stack, "Invalid value of register inside Stack!");
    }

    #[test]
    fn test_php_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        let stack_pointer_buf = cpu.stack_pointer;
        
        cpu.execute_php(&AddressingMode::Implicit);
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(1), "Invalid Stack pointer!");

        let status_from_stack = cpu.read(0x0100 + stack_pointer_buf as u16);

        assert_eq!(
            cpu.status.get() | CpuStatusRegisterFlags::Break as u8,
            status_from_stack,
            "Invalid value of register inside Stack!"
        );
    }

    #[test]
    fn test_pla_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.write(0x150, 0xFF);
        cpu.stack_pointer = 0x4F;
        cpu.execute_pla(&AddressingMode::Implicit);

        assert_eq!(cpu.register_a, 0xFF, "Register A should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");

        cpu.write(0x150, 0x00);
        cpu.stack_pointer = 0x4F;
        cpu.execute_pla(&AddressingMode::Implicit);

        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be set!");
    }

    #[test]
    fn test_plp_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.write(0x150, 0xFF);
        cpu.stack_pointer = 0x4F;
        cpu.execute_plp(&AddressingMode::Implicit);

        assert_eq!(cpu.status.get(), 0b1110_1111, "Status should have 0b1110_1111!");
    }

    #[test]
    fn test_rol_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.write(0x0000, 0xAA);
        cpu.execute_rol(&AddressingMode::ZeroPage);

        let zeropage_value = cpu.read(0x0000);
        let expected_result = 0xAAu8.rotate_left(1) - 0x1;
        assert_eq!(zeropage_value, expected_result, "Invalid value in ZeroPage!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unchanged!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");

        cpu.register_a = 0xAA;
        cpu.execute_rol(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0xAAu8.rotate_left(1), "Invalid value in Register A!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_ror_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.write(0x0000, 0xAA);
        cpu.execute_ror(&AddressingMode::ZeroPage);

        let zeropage_value = cpu.read(0x0000);
        assert_eq!(zeropage_value, 0xAAu8.rotate_left(1), "Invalid value in ZeroPage!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unchanged!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");

        cpu.register_a = 0xAA;
        cpu.execute_ror(&AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0xAAu8.rotate_left(1), "Invalid value in Register A!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_rti_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.write(0x0152, 0xFF);
        cpu.write(0x0151, 0xAB);
        cpu.write(0x0150, 0b1010_1010);
        cpu.stack_pointer = 0x4F;
        cpu.execute_rti(&AddressingMode::Implicit);

        assert_eq!(cpu.program_counter, 0xFFAB, "Program counter should have 0xAAFF!");
        assert_eq!(cpu.status.get(), 0b1010_1010, "Status should have 0b10101010!");
    }

    #[test]
    fn test_rts_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.write(0x0151, 0xFF);
        cpu.write(0x0150, 0xAA);
        cpu.stack_pointer = 0x4F;
        cpu.execute_rts(&AddressingMode::Implicit);

        assert_eq!(cpu.program_counter, 0xFFAB, "Program counter should have 0xAAFF!");
    }

    #[test]
    fn test_sbc_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0x01;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0x80);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sbc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x80, "Register A should have 0x80!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");

        cpu.register_a = 0x80;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0x01);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sbc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x7E, "Register A should have 0x80!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_sec_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sec(&AddressingMode::Implicit);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
    }

    #[test]
    fn test_sed_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.status.set_flag(CpuStatusRegisterFlags::DecimalMode, false);
        cpu.execute_sed(&AddressingMode::Implicit);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::DecimalMode), "Decimal mode flag should be set!");
    }

    #[test]
    fn test_sei_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, false);
        cpu.execute_sei(&AddressingMode::Implicit);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::InterruptDisable), "Interrupt disable flag should be set!");
    }

    #[test]
    fn test_sta_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_sta(&AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_stx_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_x = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_stx(&AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_sty_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_y = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_sty(&AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_tax_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0xFF;
        cpu.execute_tax(&AddressingMode::Implicit);

        assert_eq!(cpu.register_x, 0xFF, "Register X should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_a = 0x00;
        cpu.execute_tax(&AddressingMode::Implicit);
        assert_eq!(cpu.register_x, 0x00, "Register X should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_tay_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_a = 0xFF;
        cpu.execute_tay(&AddressingMode::Implicit);

        assert_eq!(cpu.register_y, 0xFF, "Register Y should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_a = 0x00;
        cpu.execute_tay(&AddressingMode::Implicit);
        assert_eq!(cpu.register_y, 0x00, "Register Y should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_tsx_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.stack_pointer = 0xAB;
        cpu.execute_tsx(&AddressingMode::Implicit);

        assert_eq!(cpu.register_x, 0xAB, "Register X should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.stack_pointer = 0x00;
        cpu.execute_tsx(&AddressingMode::Implicit);
        assert_eq!(cpu.register_x, 0x00, "Register X should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_txa_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_x = 0xAB;
        cpu.execute_txa(&AddressingMode::Implicit);

        assert_eq!(cpu.register_a, 0xAB, "Register A should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_x = 0x00;
        cpu.execute_txa(&AddressingMode::Implicit);
        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_txs_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_x = 0xAB;
        cpu.execute_txs(&AddressingMode::Implicit);

        assert_eq!(cpu.stack_pointer, 0xAB, "Stack pointer should have 0xAB!");
    }

    #[test]
    fn test_tya_instruction() {
        let cartridge = Cartridge::empty();
		let bus = Rc::new(RefCell::new(Bus::new(&cartridge)));
		let clock = Rc::new(RefCell::new(Clock::new()));
		let mut cpu = Cpu::new(&bus, &clock);

        cpu.register_y = 0xAB;
        cpu.execute_tya(&AddressingMode::Implicit);

        assert_eq!(cpu.register_a, 0xAB, "Register A should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_y = 0x00;
        cpu.execute_tya(&AddressingMode::Implicit);
        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }
}

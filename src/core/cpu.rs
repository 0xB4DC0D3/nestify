use std::cell::RefCell;
use std::rc::Rc;

use super::bus::Bus;
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
    bus: Rc<RefCell<Bus>>,
    internal_state: Option<InternalState>,
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

impl Cpu {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        Self {
            register_a: 0x00,
            register_x: 0x00,
            register_y: 0x00,
            status: CpuStatusRegister::new(),
            stack_pointer: 0xFD,
            program_counter: 0x8000,
            bus: bus.clone(),
            internal_state: None,
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
        // TODO: add here +7 cycles
    }

    fn is_page_cross(&self, page1: u16, page2: u16) -> bool {
        (page1 & 0xFF00) != (page2 & 0xFF00)
    }

    pub fn get_memory_data(&self, addressing_mode: AddressingMode) -> Option<(u16, bool)> {
        match addressing_mode {
            AddressingMode::Implicit => {
                None
            },
            AddressingMode::Accumulator => {
                None
            },
            AddressingMode::Immediate => {
                Some((self.program_counter, false))
            },
            AddressingMode::ZeroPage => {
                Some((self.read(self.program_counter) as u16, false))
            },
            AddressingMode::ZeroPageX => {
                let pointer = self
                    .read(self.program_counter);

                let memory_pointer = pointer 
                    .wrapping_add(self.register_x);

                let is_page_cross = self.is_page_cross(pointer as u16, memory_pointer as u16);

                Some((memory_pointer as u16, is_page_cross))
            },
            AddressingMode::ZeroPageY => {
                let pointer = self
                    .read(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_y);

                let is_page_cross = self.is_page_cross(pointer as u16, memory_pointer as u16);

                Some((memory_pointer as u16, is_page_cross))
            },
            AddressingMode::Relative => {
                Some((self.read(self.program_counter) as u16, false))
            },
            AddressingMode::Absolute => {
                Some((self.read_u16(self.program_counter), false))
            },
            AddressingMode::AbsoluteX => {
                let pointer = self
                    .read_u16(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_x as u16);

                let is_page_cross = self.is_page_cross(pointer, memory_pointer);

                Some((memory_pointer, is_page_cross))
            },
            AddressingMode::AbsoluteY => {
                let pointer = self
                    .read_u16(self.program_counter);

                let memory_pointer = pointer
                    .wrapping_add(self.register_y as u16);

                let is_page_cross = self.is_page_cross(pointer, memory_pointer);

                Some((memory_pointer, is_page_cross))
            },
            AddressingMode::Indirect => {
                let pointer = self.read_u16(self.program_counter);
                let memory_pointer = self.read_u16(pointer);

                Some((memory_pointer, false))
            },
            AddressingMode::IndexedIndirect => {
                let pointer = self
                    .read(self.program_counter)
                    .wrapping_add(self.register_x) as u16;

                let memory_pointer = self.read_u16(pointer);

                Some((memory_pointer, false))
            },
            AddressingMode::IndirectIndexed => {
                let pointer = self
                    .read(self.program_counter) as u16;

                let deref_pointer = self
                    .read_u16(pointer);

                let memory_pointer = deref_pointer
                    .wrapping_add(self.register_y as u16);

                let is_page_cross = self.is_page_cross(deref_pointer, memory_pointer as u16);

                Some((memory_pointer, is_page_cross))
            },
        }
    }

    fn execute_adc(&mut self, addressing_mode: AddressingMode) {
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
            // TODO: add additional cycle
        }
    }

    fn execute_and(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for AND instruction!");

        let result = self.register_a & self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            // TODO: add additional cycle
        }
    }

    fn execute_asl(&mut self, addressing_mode: AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value << 2;

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.register_a = result;
        }

        // TODO: add ticks
    }

    fn branch(&mut self, flag_active: bool) {
        if flag_active {
            // TODO: add +1 cycle

            let (memory_pointer, _) = self.get_memory_data(AddressingMode::Relative)
                .expect("Invalid Addressing mode for branch instructions!");

            let offset = memory_pointer as i16;
            let next_pc = self.program_counter.wrapping_add(1);
            let jump_pc = (next_pc as i16).wrapping_add(offset) as u16;

            if self.is_page_cross(next_pc, jump_pc) {
                // TODO: add +1 cycle
            }

            self.program_counter = jump_pc;
        } else {
            self.program_counter = self.program_counter.wrapping_add(1);
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

    fn execute_bit(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for BIT instruction!");

        let memory_value = self.read(memory_pointer);
        let result = self.register_a & memory_value;

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, memory_value & 0x40 == 0x40);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);

        // TODO: add ticks
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

    fn execute_brk(&mut self) {
        // Do nothing
    }

    fn execute_bvc(&mut self) {
        self.branch(!self.status.get_flag(CpuStatusRegisterFlags::Overflow));
    }

    fn execute_bvs(&mut self) {
        self.branch(self.status.get_flag(CpuStatusRegisterFlags::Overflow));
    }

    fn execute_clc(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Carry, false);

        // TODO: add ticks
    }

    fn execute_cld(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::DecimalMode, false);

        // TODO: add ticks
    }

    fn execute_cli(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, false);

        // TODO: add ticks
    }

    fn execute_clv(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Overflow, false);

        // TODO: add ticks
    }

    fn compare(&mut self, addressing_mode: AddressingMode, register_value: u8) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for CMP/CPX/CPY instructions!");

        let memory_value = self.read(memory_pointer);
        let result = register_value.wrapping_sub(memory_value);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, register_value >= memory_value);
        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if additional_cycle {
            // TODO: handle additional cycle
        }

        // TODO: handle cycles
    }

    fn execute_cmp(&mut self, addressing_mode: AddressingMode) {
        self.compare(addressing_mode, self.register_a);

        // TODO: handle cycles
    }

    fn execute_cpx(&mut self, addressing_mode: AddressingMode) {
        self.compare(addressing_mode, self.register_x);

        // TODO: handle cycles
    }

    fn execute_cpy(&mut self, addressing_mode: AddressingMode) {
        self.compare(addressing_mode, self.register_y);

        // TODO: handle cycles
    }

    fn execute_dec(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for DEC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        // TODO: handle cycles
    }

    fn execute_dex(&mut self) {
        let result = self.register_x.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_x = result;
    }

    fn execute_dey(&mut self) {
        let result = self.register_y.wrapping_sub(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_y = result;
    }

    fn execute_eor(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for EOR instruction!");

        let result = self.register_a ^ self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            // TODO: handle additional cycle
        }

        // TODO: handle cycles
    }

    fn execute_inc(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for INC instruction!");

        let memory_value = self.read(memory_pointer);
        let result = memory_value.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.write(memory_pointer, result);

        // TODO: handle cycles
    }

    fn execute_inx(&mut self) {
        let result = self.register_x.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_x = result;
    }

    fn execute_iny(&mut self) {
        let result = self.register_y.wrapping_add(1);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_y = result;
    }

    fn execute_jmp(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for JMP instruction!");

        self.program_counter = memory_pointer;
    }

    fn execute_jsr(&mut self) {
        let (memory_pointer, _) = self.get_memory_data(AddressingMode::Absolute).unwrap();

        self.push_stack_u16(self.program_counter.wrapping_add(2));
        self.program_counter = memory_pointer;
    }

    fn execute_lda(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_a = memory_value;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_ldx(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_x = memory_value;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_ldy(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for LDA instruction!");

        let memory_value = self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, memory_value == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, memory_value & 0x80 == 0x80);
        self.register_y = memory_value;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_lsr(&mut self, addressing_mode: AddressingMode) {
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

    fn execute_nop(&self) {
        // Do nothing
    }

    fn execute_ora(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, additional_cycle) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for ORA instruction!");
        
        let result = self.register_a | self.read(memory_pointer);

        self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);
        self.register_a = result;

        if additional_cycle {
            // TODO: handle add. cycle
        }
    }

    fn execute_pha(&mut self) {
        self.push_stack(self.register_a);
    }

    fn execute_php(&mut self) {
        self.push_stack(self.status.get());
    }

    fn execute_pla(&mut self) {
        let value_from_stack = self.pop_stack();

        self.status.set_flag(CpuStatusRegisterFlags::Zero, value_from_stack == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, value_from_stack & 0x80 == 0x80);
        self.register_a = value_from_stack;
    }

    fn execute_plp(&mut self) {
        let status = self.pop_stack();

        self.status.set(status);
    }

    fn execute_rol(&mut self, addressing_mode: AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value.rotate_left(1);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x80 == 0x80);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
            self.register_a = result;
        }
    }

    fn execute_ror(&mut self, addressing_mode: AddressingMode) {
        let memory_data = self.get_memory_data(addressing_mode);
        let value = if let Some((memory_pointer, _)) = memory_data {
            self.read(memory_pointer)
        } else {
            self.register_a
        };

        let result = value.rotate_right(1);

        self.status.set_flag(CpuStatusRegisterFlags::Carry, value & 0x1 == 0x1);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, result & 0x80 == 0x80);

        if let Some((memory_pointer, _)) = memory_data {
            self.write(memory_pointer, result);
        } else {
            self.status.set_flag(CpuStatusRegisterFlags::Zero, result == 0);
            self.register_a = result;
        }
    }

    fn execute_rti(&mut self) {
        let status = self.pop_stack();
        let program_counter = self.pop_stack_u16();

        self.status.set(status);
        self.program_counter = program_counter;
    }

    fn execute_rts(&mut self) {
        self.program_counter = self.pop_stack_u16();
    }

    fn execute_sbc(&mut self, addressing_mode: AddressingMode) {
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
            // TODO: add additional cycle
        }
    }

    fn execute_sec(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Carry, true);
    }

    fn execute_sed(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::DecimalMode, true);
    }

    fn execute_sei(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, true);
    }

    fn execute_sta(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_a);
    }

    fn execute_stx(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_x);
    }

    fn execute_sty(&mut self, addressing_mode: AddressingMode) {
        let (memory_pointer, _) = self.get_memory_data(addressing_mode)
            .expect("Invalid Addressing mode for STA instruction!");

        self.write(memory_pointer, self.register_y);
    }

    fn execute_tax(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_x = self.register_a;
    }

    fn execute_tay(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_a == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_a & 0x80 == 0x80);
        self.register_y = self.register_a;
    }

    fn execute_tsx(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.stack_pointer == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.stack_pointer & 0x80 == 0x80);
        self.register_x = self.stack_pointer;
    }

    fn execute_txa(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_x == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_x & 0x80 == 0x80);
        self.register_a = self.register_x;
    }

    fn execute_txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    fn execute_tya(&mut self) {
        self.status.set_flag(CpuStatusRegisterFlags::Zero, self.register_y == 0);
        self.status.set_flag(CpuStatusRegisterFlags::Negative, self.register_y & 0x80 == 0x80);
        self.register_a = self.register_y;
    }

    pub fn fetch(&mut self) {
        let Instruction {
            opcode,
            bytes,
            name,
            cycles,
            addressing_mode
        } = INSTRUCTIONS[self.read(self.program_counter) as usize];

        let current_program_counter = {
            self.program_counter = self.program_counter.wrapping_add(1);
            self.program_counter
        };

        if let Some(_) = self.bus.borrow_mut().poll_interrupt() {
            // TODO: add interrupt handle
        }

        self.internal_state = Some(InternalState {
            current_instruction: name.to_string(),
            args_length: bytes - 1
        });

        match name {
            "ADC" => self.execute_adc(addressing_mode),
            "AND" => self.execute_and(addressing_mode),
            "ASL" => self.execute_asl(addressing_mode),
            "BCC" => self.execute_bcc(),
            "BCS" => self.execute_bcs(),
            "BEQ" => self.execute_beq(),
            "BIT" => self.execute_bit(addressing_mode),
            "BMI" => self.execute_bne(),
            "BPL" => self.execute_bpl(),
            "BRK" => self.execute_brk(),
            "BVC" => self.execute_bvc(),
            "BVS" => self.execute_bvs(),
            "CLC" => self.execute_clc(),
            "CLD" => self.execute_cld(),
            "CLV" => self.execute_clv(),
            "CMP" => self.execute_cmp(addressing_mode),
            "CPX" => self.execute_cpx(addressing_mode),
            "CPY" => self.execute_cpy(addressing_mode),
            "DEC" => self.execute_dec(addressing_mode),
            "DEX" => self.execute_dex(),
            "DEY" => self.execute_dey(),
            "EOR" => self.execute_eor(addressing_mode),
            "INC" => self.execute_inc(addressing_mode),
            "INX" => self.execute_inx(),
            "INY" => self.execute_iny(),
            "JMP" => self.execute_jmp(addressing_mode),
            "JSR" => self.execute_jsr(),
            "LDA" => self.execute_lda(addressing_mode),
            "LDX" => self.execute_ldx(addressing_mode),
            "LDY" => self.execute_ldy(addressing_mode),
            "LSR" => self.execute_lsr(addressing_mode),
            "NOP" => self.execute_nop(),
            "ORA" => self.execute_ora(addressing_mode),
            "PHA" => self.execute_pha(),
            "PHP" => self.execute_php(),
            "PLA" => self.execute_pla(),
            "PLP" => self.execute_plp(),
            "ROL" => self.execute_rol(addressing_mode),
            "ROR" => self.execute_ror(addressing_mode),
            "RTI" => self.execute_rti(),
            "RTS" => self.execute_rts(),
            "SBC" => self.execute_sbc(addressing_mode),
            "SEC" => self.execute_sec(),
            "SED" => self.execute_sed(),
            "SEI" => self.execute_sei(),
            "STA" => self.execute_sta(addressing_mode),
            "STX" => self.execute_stx(addressing_mode),
            "STY" => self.execute_sty(addressing_mode),
            "TAX" => self.execute_tax(),
            "TAY" => self.execute_tay(),
            "TSX" => self.execute_tsx(),
            "TXA" => self.execute_txa(),
            "TXS" => self.execute_txs(),
            "TYA" => self.execute_tya(),
            _ => panic!("Illegal opcode {:#02X} occured!", opcode),
        }

        if current_program_counter == self.program_counter {
            let args_length = self.internal_state.as_ref().unwrap().args_length as u16;

            self.program_counter = self.program_counter.wrapping_add(args_length);
        }
    }
}

impl Memory for Cpu {
    fn read(&self, address: u16) -> u8 {
        self.bus
            .borrow_mut()
            .get_cpu_memory_map()
            .read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus
            .borrow_mut()
            .get_cpu_memory_map()
            .write(address, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 127;
        cpu.write(0x0000, 0x69);
        cpu.write(0x0001, 0x7F);
        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);

        cpu.execute_adc(AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 254, "Register A should be 254!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow flag should be set!");

        cpu.register_a = 128;
        cpu.write(0x0000, 0x69);
        cpu.write(0x0001, 0x80);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);

        cpu.execute_adc(AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 1, "Register A should be 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow flag should be set!");
    }

    #[test]
    fn test_and_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 127;
        cpu.write(0x0000, 0x29);
        cpu.write(0x0001, 0x7E);
        cpu.program_counter = 0x0001;

        cpu.execute_and(AddressingMode::Immediate);
        assert_eq!(cpu.register_a, 0x7E, "Register A should be 126!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
    }

    #[test]
    fn test_asl_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 0x80;
        cpu.execute_asl(AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0x00, "Register A should be 0!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
    }

    #[test]
    fn test_bcc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_bcc();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BCC with inactive Carry flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_bcc();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BCC with active Carry flag!");
    }

    #[test]
    fn test_bcs_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_bcs();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BCS with inactive Carry flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_bcs();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BCS with active Carry flag!");
    }

    #[test]
    fn test_beq_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, false);
        cpu.execute_beq();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BEQ with inactive Zero flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.execute_beq();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BEQ with active Zero flag!");
    }

    #[test]
    fn test_bit_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.program_counter = 0x0001;
        cpu.write(0x0000, 0xC0);

        // 0x0001 - lobyte of 0x0000, 0x0002 - hibyte of 0x0000
        cpu.write(0x0001, 0x00);
        cpu.write(0x0002, 0x00);
        cpu.execute_bit(AddressingMode::Absolute);
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be set!");

        cpu.write(0x0000, 0x80);
        cpu.execute_bit(AddressingMode::Absolute);
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");

        cpu.write(0x0000, 0x40);
        cpu.execute_bit(AddressingMode::Absolute);
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");

        cpu.write(0x0000, 0x3F);
        cpu.execute_bit(AddressingMode::Absolute);
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "CPU Status: Overflow should be unset!");
    }

    #[test]
    fn test_bmi_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, false);
        cpu.execute_bmi();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BMI with inactive Negative flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, true);
        cpu.execute_bmi();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BMI with active negative flag!");
    }

    #[test]
    fn test_bne_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, false);
        cpu.execute_bne();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BNE with inactive Zero flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.execute_bne();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BNE with active Zero flag!");
    }

    #[test]
    fn test_bpl_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, false);
        cpu.execute_bpl();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BPL with inactive Negative flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Negative, true);
        cpu.execute_bpl();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BPL with active Negative flag!");
    }

    #[test]
    fn test_bvc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
        cpu.execute_bvc();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BVC with inactive Overflow flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_bvc();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BVC with active Overflow flag!");
    }

    #[test]
    fn test_bvs_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0001;
        cpu.write(0x0001, 0x04);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_bvs();
        assert_eq!(cpu.program_counter, 0x0006, "CPU PC should be 0x0006 after BVC with active Overflow flag!");

        cpu.program_counter = 0x0001;
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, false);
        cpu.execute_bvs();
        assert_eq!(cpu.program_counter, 0x0002, "CPU PC should be 0x0002 after BVC with inactive Overflow flag!");
    }

    #[test] 
    fn test_clc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, true);
        cpu.execute_clc();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be unset!");
    }

    #[test]
    fn test_cld_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::DecimalMode, true);
        cpu.execute_cld();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::DecimalMode), "Decimal mode flag should be unset!");
    }

    #[test]
    fn test_cli_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, true);
        cpu.execute_cli();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::InterruptDisable), "Interrupt disable flag should be unset!");
    }

    #[test]
    fn test_clv_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.status.set_flag(CpuStatusRegisterFlags::Overflow, true);
        cpu.execute_clv();

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be unset!");
    }

    #[test]
    fn test_cmp_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_a = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_a = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cmp(AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_cpx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_x = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_x = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_x = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cpx(AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_cpy_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_y = 100;
        cpu.write(0x0000, 99);
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_y = 98;
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(AddressingMode::Immediate);

        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");

        cpu.register_y = 99;
        cpu.program_counter = 0x0000;
        cpu.execute_cpy(AddressingMode::Immediate);

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set, A >= M!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset, A - M == 1!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset, A - M <= 127!");
    }

    #[test]
    fn test_dec_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.write(0x0001, 129);
        cpu.write(0x0002, 0x1);
        cpu.program_counter = 0x0002;
        cpu.execute_dec(AddressingMode::ZeroPageX);

        let memory_value = cpu.read(0x0001);
        assert_eq!(memory_value, 128, "Memory value at 0x0001 should be 128!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_dex_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_x = 128;
        cpu.execute_dex();

        assert_eq!(cpu.register_x, 127, "Register X should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_dey_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_y = 128;
        cpu.execute_dey();

        assert_eq!(cpu.register_y, 127, "Register Y should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_eor_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_a = 12;
        cpu.write(0x0000, 37);
        cpu.program_counter = 0x0000;
        cpu.execute_eor(AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 41, "Register A should contain result of 12 ^ 37!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_inc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.write(0x0001, 129);
        cpu.write(0x0002, 0x1);
        cpu.program_counter = 0x0002;
        cpu.execute_inc(AddressingMode::ZeroPageX);

        let memory_value = cpu.read(0x0001);
        assert_eq!(memory_value, 130, "Memory value at 0x0001 should be 130!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_inx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let mut cpu = Cpu::new(&bus);
        cpu.register_x = 128;
        cpu.execute_inx();

        assert_eq!(cpu.register_x, 129, "Register X should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_iny_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_y = 128;
        cpu.execute_iny();

        assert_eq!(cpu.register_y, 129, "Register Y should be 127!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_jmp_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xFF);
        cpu.write(0x0001, 0xAA);
        cpu.execute_jmp(AddressingMode::Absolute);

        assert_eq!(cpu.program_counter, 0xAAFF, "Program counter should be 0xAAFF!");
    }

    #[test]
    fn test_jsr_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        let stack_pointer_buf = cpu.stack_pointer;
        cpu.program_counter = 0xDEAD;
        cpu.write(0xDEAD, 0xFF);
        cpu.write(0xDEAE, 0xAA);
        cpu.execute_jsr();

        assert_eq!(cpu.program_counter, 0xAAFF, "Program counter should be 0xAAFF");
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(2), "Invalid stack pointer!");

        let lo = cpu.read(0x0100 + cpu.stack_pointer.wrapping_add(1) as u16);
        let hi = cpu.read(0x0100 + cpu.stack_pointer.wrapping_add(2) as u16);
        assert_eq!(lo, 0xAF, "Invalid lobyte of PC in Stack!");
        assert_eq!(hi, 0xDE, "Invalid hibyte of PC in Stack!");
    }

    #[test]
    fn test_lda_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_lda(AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_lda(AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_ldx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_ldx(AddressingMode::Immediate);

        assert_eq!(cpu.register_x, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_ldx(AddressingMode::Immediate);

        assert_eq!(cpu.register_x, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_ldy_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xAA);
        cpu.execute_ldy(AddressingMode::Immediate);

        assert_eq!(cpu.register_y, 0xAA, "Register A should be 0xAA!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.write(0x0000, 0x00);
        cpu.execute_ldy(AddressingMode::Immediate);

        assert_eq!(cpu.register_y, 0x00, "Register A should be 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_lsr_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0x81;
        cpu.execute_lsr(AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0x40, "Register A should be 0x40!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "CPU Status: Negative flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "CPU Status: Zero should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "CPU Status: Carry should be set!");
    }

    #[test]
    fn test_ora_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0x81;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0xFF);
        cpu.execute_ora(AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0xFF, "Register A should be 0xFF!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_pha_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        let stack_pointer_buf = cpu.stack_pointer;

        cpu.execute_pha();
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(1), "Invalid Stack pointer!");

        let register_from_stack = cpu.read(0x0100 + stack_pointer_buf as u16);
        assert_eq!(cpu.register_a, register_from_stack, "Invalid value of register inside Stack!");
    }

    #[test]
    fn test_php_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        let stack_pointer_buf = cpu.stack_pointer;
        
        cpu.execute_php();
        assert_eq!(cpu.stack_pointer, stack_pointer_buf.wrapping_sub(1), "Invalid Stack pointer!");

        let status_from_stack = cpu.read(0x0100 + stack_pointer_buf as u16);
        assert_eq!(cpu.status.get(), status_from_stack, "Invalid value of register inside Stack!");
    }

    #[test]
    fn test_pla_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x150, 0xFF);
        cpu.stack_pointer = 0x4F;
        cpu.execute_pla();

        assert_eq!(cpu.register_a, 0xFF, "Register A should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");

        cpu.write(0x150, 0x00);
        cpu.stack_pointer = 0x4F;
        cpu.execute_pla();

        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be set!");
    }

    #[test]
    fn test_plp_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x150, 0xFF);
        cpu.stack_pointer = 0x4F;
        cpu.execute_plp();

        assert_eq!(cpu.status.get(), 0xFF, "Status should have 0xFF!");
    }

    #[test]
    fn test_rol_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.write(0x0000, 0xAA);
        cpu.execute_rol(AddressingMode::ZeroPage);

        let zeropage_value = cpu.read(0x0000);
        assert_eq!(zeropage_value, 0xAAu8.rotate_left(1), "Invalid value in ZeroPage!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unchanged!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");

        cpu.register_a = 0xAA;
        cpu.execute_rol(AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0xAAu8.rotate_left(1), "Invalid value in Register A!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_ror_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::Zero, true);
        cpu.write(0x0000, 0xAA);
        cpu.execute_ror(AddressingMode::ZeroPage);

        let zeropage_value = cpu.read(0x0000);
        assert_eq!(zeropage_value, 0xAAu8.rotate_left(1), "Invalid value in ZeroPage!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unchanged!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");

        cpu.register_a = 0xAA;
        cpu.execute_ror(AddressingMode::Accumulator);

        assert_eq!(cpu.register_a, 0xAAu8.rotate_left(1), "Invalid value in Register A!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
    }

    #[test]
    fn test_rti_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x0152, 0xFF);
        cpu.write(0x0151, 0xAB);
        cpu.write(0x0150, 0b1010_1010);
        cpu.stack_pointer = 0x4F;
        cpu.execute_rti();

        assert_eq!(cpu.program_counter, 0xFFAB, "Program counter should have 0xAAFF!");
        assert_eq!(cpu.status.get(), 0b1010_1010, "Status should have 0b10101010!");
    }

    #[test]
    fn test_rts_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.write(0x0151, 0xFF);
        cpu.write(0x0150, 0xAA);
        cpu.stack_pointer = 0x4F;
        cpu.execute_rts();

        assert_eq!(cpu.program_counter, 0xFFAA, "Program counter should have 0xAAFF!");
    }

    #[test]
    fn test_sbc_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0x01;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0x80);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sbc(AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x80, "Register A should have 0x80!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be set!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");

        cpu.register_a = 0x80;
        cpu.program_counter = 0x0000;
        cpu.write(0x0000, 0x01);
        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sbc(AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x7E, "Register A should have 0x80!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Zero flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Overflow), "Overflow flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
    }

    #[test]
    fn test_sec_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::Carry, false);
        cpu.execute_sec();

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Carry), "Carry flag should be set!");
    }

    #[test]
    fn test_sed_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::DecimalMode, false);
        cpu.execute_sed();

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::DecimalMode), "Decimal mode flag should be set!");
    }

    #[test]
    fn test_sei_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.status.set_flag(CpuStatusRegisterFlags::InterruptDisable, false);
        cpu.execute_sei();

        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::InterruptDisable), "Interrupt disable flag should be set!");
    }

    #[test]
    fn test_sta_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_sta(AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_stx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_x = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_stx(AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_sty_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_y = 0xFF;
        cpu.write(0x0001, 0x00);
        cpu.program_counter = 0x0001;
        cpu.execute_sty(AddressingMode::ZeroPage);

        let result = cpu.read(0x0000);
        assert_eq!(result, 0xFF, "Zero page value should have 0xFF!");
    }

    #[test]
    fn test_tax_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.execute_tax();

        assert_eq!(cpu.register_x, 0xFF, "Register X should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_a = 0x00;
        cpu.execute_tax();
        assert_eq!(cpu.register_x, 0x00, "Register X should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_tay_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_a = 0xFF;
        cpu.execute_tay();

        assert_eq!(cpu.register_y, 0xFF, "Register Y should have 0xFF!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_a = 0x00;
        cpu.execute_tay();
        assert_eq!(cpu.register_y, 0x00, "Register Y should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_tsx_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.stack_pointer = 0xAB;
        cpu.execute_tsx();

        assert_eq!(cpu.register_x, 0xAB, "Register X should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.stack_pointer = 0x00;
        cpu.execute_tsx();
        assert_eq!(cpu.register_x, 0x00, "Register X should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_txa_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_x = 0xAB;
        cpu.execute_txa();

        assert_eq!(cpu.register_a, 0xAB, "Register A should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_x = 0x00;
        cpu.execute_txa();
        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }

    #[test]
    fn test_txs_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_x = 0xAB;
        cpu.execute_txs();

        assert_eq!(cpu.stack_pointer, 0xAB, "Stack pointer should have 0xAB!");
    }

    #[test]
    fn test_tya_instruction() {
        let bus = Rc::new(RefCell::new(Bus::new()));
		let mut cpu = Cpu::new(&bus);

        cpu.register_y = 0xAB;
        cpu.execute_tya();

        assert_eq!(cpu.register_a, 0xAB, "Register A should have 0xAB!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be set!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be unset!");

        cpu.register_y = 0x00;
        cpu.execute_tya();
        assert_eq!(cpu.register_a, 0x00, "Register A should have 0x00!");
        assert!(!cpu.status.get_flag(CpuStatusRegisterFlags::Negative), "Negative flag should be unset!");
        assert!(cpu.status.get_flag(CpuStatusRegisterFlags::Zero), "Negative flag should be set!");
    }
}
